use std::ops::{BitOr, BitOrAssign};

use constant_propagation_map::ConstantPropagationMap;

use crate::ast::{Expression, ExpressionKind, Function, Statement, StatementKind, SymbolId};

use super::CompileSettings;

mod constant_propagation_map;

#[derive(Clone, Copy, Debug)]
enum Constant {
    Int(i32),
    Fix(agb_fixnum::Num<i32, 8>),
    Bool(bool),
}

impl From<Constant> for ExpressionKind<'_> {
    fn from(value: Constant) -> Self {
        match value {
            Constant::Int(i) => ExpressionKind::Integer(i),
            Constant::Fix(num) => ExpressionKind::Fix(num),
            Constant::Bool(b) => ExpressionKind::Bool(b),
        }
    }
}

impl TryFrom<&ExpressionKind<'_>> for Constant {
    type Error = ();

    fn try_from(value: &ExpressionKind<'_>) -> Result<Self, Self::Error> {
        Ok(match value {
            ExpressionKind::Integer(i) => Constant::Int(*i),
            ExpressionKind::Fix(num) => Constant::Fix(*num),
            ExpressionKind::Bool(b) => Constant::Bool(*b),
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConstantPropagationResult {
    DidSomething,
    DidNothing,
}

impl BitOrAssign for ConstantPropagationResult {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitOr for ConstantPropagationResult {
    type Output = ConstantPropagationResult;

    fn bitor(self, rhs: Self) -> Self::Output {
        if self == ConstantPropagationResult::DidSomething
            || rhs == ConstantPropagationResult::DidSomething
        {
            ConstantPropagationResult::DidSomething
        } else {
            ConstantPropagationResult::DidNothing
        }
    }
}

fn constant_propagation(
    function: &mut Function,
    compile_settings: &CompileSettings,
) -> ConstantPropagationResult {
    let mut constant_symbol = ConstantPropagationMap::default();
    constant_propagation_block(
        &mut function.statements,
        &mut constant_symbol,
        compile_settings,
    )
}

fn constant_propagation_block(
    block: &mut [Statement],
    constant_symbols: &mut ConstantPropagationMap,
    compile_settings: &CompileSettings,
) -> ConstantPropagationResult {
    block
        .iter_mut()
        .map(|statement| match &mut statement.kind {
            StatementKind::Error
            | StatementKind::Continue
            | StatementKind::Break
            | StatementKind::Nop => ConstantPropagationResult::DidNothing,
            StatementKind::Wait => {
                constant_symbols.poison_properties(compile_settings);

                ConstantPropagationResult::DidNothing
            }
            StatementKind::Assignment { value, .. }
            | StatementKind::VariableDeclaration { value, .. } => {
                let did_propagate = constant_propagation_expr(value, constant_symbols);

                let symbol_id = statement.meta.get().unwrap();
                constant_symbols.set(*symbol_id, &value.kind);

                did_propagate
            }
            StatementKind::If {
                condition,
                true_block,
                false_block,
            } => {
                let mut did_propagate = constant_propagation_expr(condition, constant_symbols);
                let mut true_block_symbols = constant_symbols.snapshot();
                did_propagate |= constant_propagation_block(
                    true_block,
                    &mut true_block_symbols,
                    compile_settings,
                );
                let mut false_block_symbols = constant_symbols.snapshot();
                did_propagate |= constant_propagation_block(
                    false_block,
                    &mut false_block_symbols,
                    compile_settings,
                );

                constant_symbols.apply_poisons(&true_block_symbols);
                constant_symbols.apply_poisons(&false_block_symbols);

                did_propagate
            }
            StatementKind::Loop { block } => todo!(),
            StatementKind::Call { name, arguments } => todo!(),
            StatementKind::Spawn { name, arguments } => todo!(),
            StatementKind::Return { values } => todo!(),
        })
        .reduce(BitOr::bitor)
        .unwrap_or(ConstantPropagationResult::DidNothing)
}

fn constant_propagation_expr(
    expression: &mut Expression,
    constant_symbols: &ConstantPropagationMap,
) -> ConstantPropagationResult {
    match &mut expression.kind {
        ExpressionKind::Integer(_)
        | ExpressionKind::Fix(_)
        | ExpressionKind::Bool(_)
        | ExpressionKind::Error
        | ExpressionKind::Nop => ConstantPropagationResult::DidNothing,
        ExpressionKind::BinaryOperation {
            ref mut lhs,
            ref mut rhs,
            ..
        } => {
            constant_propagation_expr(lhs, constant_symbols)
                | constant_propagation_expr(rhs, constant_symbols)
        }
        ExpressionKind::Call { arguments, .. } => arguments
            .iter_mut()
            .map(|expr| constant_propagation_expr(expr, constant_symbols))
            .reduce(BitOr::bitor)
            .unwrap_or(ConstantPropagationResult::DidNothing),
        ExpressionKind::Variable(_) => {
            let symbol = expression.meta.get().expect("Variable should have symbol");
            if let Some(constant) = constant_symbols.get(*symbol) {
                expression.kind = constant.into();
                ConstantPropagationResult::DidSomething
            } else {
                ConstantPropagationResult::DidNothing
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use insta::{assert_ron_snapshot, glob};

    use crate::{
        compile::{
            loop_visitor::visit_loop_check, symtab_visitor::SymTabVisitor,
            type_visitor::TypeVisitor,
        },
        grammar,
        lexer::Lexer,
        reporting::Diagnostics,
        tokens::FileId,
        CompileSettings, Property, Type,
    };

    use super::*;

    #[test]
    fn constant_propagation_snapshot_tests() {
        glob!("snapshot_tests", "constant_propagation/*.tapir", |path| {
            let input = fs::read_to_string(path).unwrap();

            let lexer = Lexer::new(&input, FileId::new(0));
            let parser = grammar::ScriptParser::new();
            let file_id = FileId::new(0);

            let mut diagnostics = Diagnostics::new(file_id, path.file_name().unwrap(), &input);

            let mut script = parser.parse(file_id, &mut diagnostics, lexer).unwrap();

            let compile_settings = CompileSettings {
                properties: vec![Property {
                    ty: Type::Int,
                    index: 0,
                    name: "int_prop".to_owned(),
                }],
            };

            let mut symtab_visitor = SymTabVisitor::new(&compile_settings);
            let mut type_visitor =
                TypeVisitor::new(&compile_settings, &script.functions, &mut diagnostics);

            for function in &mut script.functions {
                visit_loop_check(function, &mut diagnostics);
                symtab_visitor.visit_function(function, &mut diagnostics);
                type_visitor.visit_function(
                    function,
                    symtab_visitor.get_symtab(),
                    &mut diagnostics,
                );

                constant_propagation(function, &compile_settings);
            }

            assert_ron_snapshot!(script, {
                ".**.span" => "[span]",
                ".**.meta" => "[meta]",
            });
        });
    }
}
