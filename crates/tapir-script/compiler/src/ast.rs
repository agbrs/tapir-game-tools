use std::iter;

use crate::{
    tokens::{FileId, Span},
    types::Type,
};

use metadata::Metadata;

use serde::Serialize;

mod metadata;
#[cfg(test)]
mod pretty_printer;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize)]
pub struct SymbolId(pub usize);

pub type Fix = agb_fixnum::Num<i32, 8>;

#[derive(Clone, Debug, Serialize)]
pub struct Script<'input> {
    pub functions: Vec<Function<'input>>,
}

impl<'input> Script<'input> {
    pub fn from_top_level(
        top_level: impl IntoIterator<Item = TopLevelStatement<'input>>,
        file_id: FileId,
    ) -> Self {
        let mut top_level_function_statements = vec![];
        let mut functions = vec![];

        for top_level_statement in top_level.into_iter() {
            match top_level_statement {
                TopLevelStatement::Statement(statement) => {
                    top_level_function_statements.push(statement)
                }
                TopLevelStatement::FunctionDefinition(function) => functions.push(function),

                TopLevelStatement::Error => {}
            }
        }

        let top_level_function = Function {
            name: "@toplevel",
            span: Span::new(file_id, 0, 0),
            statements: top_level_function_statements,
            arguments: vec![],
            return_types: FunctionReturn {
                types: vec![],
                span: Span::new(file_id, 0, 0),
            },
            modifiers: FunctionModifiers::default(),
        };

        functions.insert(0, top_level_function);

        Self { functions }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Function<'input> {
    pub name: &'input str,
    pub span: Span,
    pub statements: Vec<Statement<'input>>,
    pub arguments: Vec<FunctionArgument<'input>>,
    pub return_types: FunctionReturn,

    pub modifiers: FunctionModifiers,
}

impl Script<'_> {
    #[cfg(test)]
    pub fn pretty_print(&self) -> String {
        let mut output = String::new();

        for function in &self.functions {
            pretty_printer::pretty_print(function, &mut output).unwrap();
        }

        output
    }
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct FunctionModifiers {
    pub is_event_handler: Option<Span>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FunctionReturn {
    pub types: Vec<TypeWithLocation>,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize)]
pub struct TypeWithLocation {
    pub t: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize)]
pub struct FunctionArgument<'input> {
    pub span: Span,
    pub t: TypeWithLocation,
    pub name: MaybeResolved<'input>,
}

#[derive(Clone, Debug, Serialize)]
pub enum MaybeResolved<'input> {
    Unresolved(&'input str),
    Resolved(SymbolId),
}

#[derive(Clone, Debug, Serialize)]
pub enum TopLevelStatement<'input> {
    Statement(Statement<'input>),
    FunctionDefinition(Function<'input>),
    Error,
}

#[derive(Clone, Debug, Serialize)]
pub struct Statement<'input> {
    pub span: Span,
    pub kind: StatementKind<'input>,

    pub meta: Metadata,
}

impl<'input> Statement<'input> {
    pub(crate) fn expressions_mut(
        &mut self,
    ) -> Box<dyn Iterator<Item = &mut Expression<'input>> + '_> {
        match &mut self.kind {
            StatementKind::Error
            | StatementKind::Wait
            | StatementKind::Continue
            | StatementKind::Break
            | StatementKind::Nop => Box::new(iter::empty()),
            StatementKind::VariableDeclaration { value, .. }
            | StatementKind::Assignment { value, .. } => Box::new(iter::once(value)),
            StatementKind::If {
                condition,
                true_block,
                false_block,
            } => Box::new(
                iter::once(condition)
                    .chain(true_block.iter_mut().flat_map(Statement::expressions_mut))
                    .chain(false_block.iter_mut().flat_map(Statement::expressions_mut)),
            ),
            StatementKind::Block { block } | StatementKind::Loop { block } => {
                Box::new(block.iter_mut().flat_map(Statement::expressions_mut))
            }
            StatementKind::Call { arguments, .. }
            | StatementKind::Return { values: arguments }
            | StatementKind::Spawn { arguments, .. } => Box::new(arguments.iter_mut()),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub enum StatementKind<'input> {
    Error,
    VariableDeclaration {
        ident: &'input str,
        value: Expression<'input>,
    },
    Assignment {
        ident: &'input str,
        value: Expression<'input>,
    },
    Wait,
    Block {
        block: Vec<Statement<'input>>,
    },
    Continue,
    Break,
    #[default]
    Nop,

    If {
        condition: Expression<'input>,
        true_block: Vec<Statement<'input>>,
        false_block: Vec<Statement<'input>>,
    },
    Loop {
        block: Vec<Statement<'input>>,
    },

    Call {
        name: &'input str,
        arguments: Vec<Expression<'input>>,
    },
    Spawn {
        name: &'input str,
        arguments: Vec<Expression<'input>>,
    },
    Return {
        values: Vec<Expression<'input>>,
    },
}

impl<'input> StatementKind<'input> {
    pub fn with_span(self, file_id: FileId, start: usize, end: usize) -> Statement<'input> {
        Statement {
            span: Span::new(file_id, start, end),
            kind: self,
            meta: Metadata::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Expression<'input> {
    pub span: Span,
    pub kind: ExpressionKind<'input>,

    pub meta: Metadata,
}

#[derive(Clone, Default, Debug, Serialize)]
pub enum ExpressionKind<'input> {
    Integer(i32),
    Fix(#[serde(skip)] Fix),
    Bool(bool),
    Variable(&'input str),
    BinaryOperation {
        lhs: Box<Expression<'input>>,
        operator: BinaryOperator,
        rhs: Box<Expression<'input>>,
    },
    Error,
    #[default]
    Nop,

    Call {
        name: &'input str,
        arguments: Vec<Expression<'input>>,
    },
}

impl<'input> ExpressionKind<'input> {
    pub fn with_span(self, file_id: FileId, start: usize, end: usize) -> Expression<'input> {
        Expression {
            kind: self,
            span: Span::new(file_id, start, end),

            meta: Metadata::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    RealDiv,
    RealMod,

    FixMul,
    FixDiv,

    EqEq,
    NeEq,
    Gt,
    GtEq,
    Lt,
    LtEq,

    Then,
}

impl BinaryOperator {
    pub fn update_type_with_lhs(&mut self, lhs_type: Type) {
        use BinaryOperator as B;
        match (*self, lhs_type) {
            (B::Mul, Type::Fix) => *self = B::FixMul,
            (B::Div, Type::Fix) => *self = B::FixDiv,
            _ => {}
        }
    }

    pub fn can_handle_type(self, lhs_type: Type) -> bool {
        use BinaryOperator as B;
        match self {
            B::Add
            | B::Sub
            | B::Mul
            | B::Div
            | B::Mod
            | B::RealDiv
            | B::RealMod
            | B::Gt
            | B::GtEq
            | B::Lt
            | B::LtEq => {
                matches!(lhs_type, Type::Fix | Type::Int)
            }

            B::FixMul | B::FixDiv => matches!(lhs_type, Type::Fix),

            B::EqEq | B::NeEq => !matches!(lhs_type, Type::Error),
            B::Then => true,
        }
    }

    pub fn resulting_type(self, lhs_type: Type, rhs_type: Type) -> Type {
        use BinaryOperator as B;

        match self {
            B::Add
            | B::Sub
            | B::Mul
            | B::Div
            | B::Mod
            | B::RealDiv
            | B::RealMod
            | B::FixMul
            | B::FixDiv => lhs_type,

            B::EqEq | B::NeEq | B::Gt | B::GtEq | B::Lt | B::LtEq => Type::Bool,
            B::Then => rhs_type,
        }
    }
}
