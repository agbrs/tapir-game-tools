use crate::{
    tokens::{FileId, Span},
    types::Type,
};

use serde::Serialize;

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
    #[default]
    Nop,

    If {
        condition: Expression<'input>,
        true_block: Vec<Statement<'input>>,
        false_block: Vec<Statement<'input>>,
    },

    // From here these can only be generated by the compiler
    SymbolDeclare {
        ident: SymbolId,
        value: Expression<'input>,
    },
    SymbolAssign {
        ident: SymbolId,
        value: Expression<'input>,
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
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Expression<'input> {
    pub span: Span,
    pub kind: ExpressionKind<'input>,
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

    // From here these can only be generated by the compiler
    Symbol(SymbolId),
}

impl<'input> ExpressionKind<'input> {
    pub fn with_span(self, file_id: FileId, start: usize, end: usize) -> Expression<'input> {
        Expression {
            kind: self,
            span: Span::new(file_id, start, end),
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

    EqEq,
    NeEq,
    Gt,
    GtEq,
    Lt,
    LtEq,
}

impl BinaryOperator {
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

            B::EqEq | B::NeEq => !matches!(lhs_type, Type::Error),
        }
    }

    pub fn resulting_type(self, lhs_type: Type) -> Type {
        use BinaryOperator as B;

        match self {
            B::Add | B::Sub | B::Mul | B::Div | B::Mod | B::RealDiv | B::RealMod => lhs_type,

            B::EqEq | B::NeEq | B::Gt | B::GtEq | B::Lt | B::LtEq => Type::Bool,
        }
    }
}
