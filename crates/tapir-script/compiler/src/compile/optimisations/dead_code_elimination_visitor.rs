use std::collections::HashSet;

use crate::{
    ast::{Expression, ExpressionKind, Function, Statement, StatementKind, SymbolId},
    CompileSettings,
};

#[derive(Default, Clone)]
struct UsedAssignmentsSet {
    used: HashSet<SymbolId>,
}

impl UsedAssignmentsSet {
    fn poison_properties(&mut self, compile_settings: &CompileSettings) {
        self.used.extend(compile_settings.property_symbols());
    }

    fn used(&mut self, symbol: SymbolId) {
        self.used.insert(symbol);
    }

    fn remove(&mut self, symbol: SymbolId) -> bool {
        self.used.remove(&symbol)
    }

    fn combine(self, other: UsedAssignmentsSet) -> UsedAssignmentsSet {
        UsedAssignmentsSet {
            used: self.used.union(&other.used).copied().collect(),
        }
    }

    fn absorb(&mut self, other: UsedAssignmentsSet) {
        self.used.extend(other.used);
    }
}

pub fn dead_code_eliminate(function: &mut Function, compile_settings: &CompileSettings) {
    eliminate_after_control_flow_diverge(&mut function.statements);

    let mut constant_symbol = UsedAssignmentsSet::default();
    constant_symbol.poison_properties(compile_settings);
    annotate_dead_statements(
        &mut function.statements,
        &mut constant_symbol,
        compile_settings,
        true,
    );

    sweep_dead_statements(&mut function.statements);
}

fn eliminate_after_control_flow_diverge(block: &mut Vec<Statement>) {
    let mut encountered_control_flow_diverge = false;
    block.retain_mut(|statement| {
        if encountered_control_flow_diverge {
            return false;
        }
        match &mut statement.kind {
            StatementKind::Continue | StatementKind::Break => {
                encountered_control_flow_diverge = true;
            }
            StatementKind::Error
            | StatementKind::Nop
            | StatementKind::Wait
            | StatementKind::Assignment { .. }
            | StatementKind::VariableDeclaration { .. }
            | StatementKind::Call { .. }
            | StatementKind::Spawn { .. }
            | StatementKind::Return { .. } => {}
            StatementKind::If {
                true_block,
                false_block,
                ..
            } => {
                eliminate_after_control_flow_diverge(true_block);
                eliminate_after_control_flow_diverge(false_block);
            }
            StatementKind::Loop { block } => {
                eliminate_after_control_flow_diverge(block);
            }
        }
        true
    });
}

fn sweep_dead_statements(block: &mut Vec<Statement>) {
    block.retain_mut(|statement| {
        match &mut statement.kind {
            StatementKind::Continue
            | StatementKind::Break
            | StatementKind::Error
            | StatementKind::Nop
            | StatementKind::Wait
            | StatementKind::Assignment { .. }
            | StatementKind::VariableDeclaration { .. }
            | StatementKind::Call { .. }
            | StatementKind::Spawn { .. }
            | StatementKind::Return { .. } => {}
            StatementKind::If {
                true_block,
                false_block,
                ..
            } => {
                sweep_dead_statements(true_block);
                sweep_dead_statements(false_block);
            }
            StatementKind::Loop { block } => {
                sweep_dead_statements(block);
            }
        }
        statement.meta.get::<DeadStatement>().is_none()
    });
}

#[derive(Debug)]
struct DeadStatement;

fn annotate_dead_statements(
    block: &mut [Statement],
    used_symbols: &mut UsedAssignmentsSet,
    compile_settings: &CompileSettings,
    mark_as_dead: bool,
) {
    for statement in block.iter_mut().rev() {
        match &mut statement.kind {
            StatementKind::Error
            | StatementKind::Continue
            | StatementKind::Break
            | StatementKind::Nop => {}
            StatementKind::Wait => {
                used_symbols.poison_properties(compile_settings);
            }
            StatementKind::Assignment { value, .. }
            | StatementKind::VariableDeclaration { value, .. } => {
                let symbol = statement.meta.get().unwrap();
                let symbol_is_used = used_symbols.remove(*symbol);
                if !symbol_is_used {
                    if mark_as_dead {
                        statement.meta.set(DeadStatement);
                    }
                } else {
                    dead_code_visit_expression(value, used_symbols, compile_settings);
                }
            }
            StatementKind::If {
                true_block,
                false_block,
                condition,
            } => {
                let mut true_block_used_symbols = used_symbols.clone();
                let mut false_block_used_symbols = used_symbols.clone();
                annotate_dead_statements(
                    true_block,
                    &mut true_block_used_symbols,
                    compile_settings,
                    true,
                );
                annotate_dead_statements(
                    false_block,
                    &mut false_block_used_symbols,
                    compile_settings,
                    true,
                );
                *used_symbols = true_block_used_symbols.combine(false_block_used_symbols);
                dead_code_visit_expression(condition, used_symbols, compile_settings);
            }
            StatementKind::Loop { block } => {
                let mut analyse_existing = used_symbols.clone();
                annotate_dead_statements(block, &mut analyse_existing, compile_settings, false);
                used_symbols.absorb(analyse_existing);
                annotate_dead_statements(block, used_symbols, compile_settings, true);
            }
            StatementKind::Call { arguments, .. } => {
                for expr in arguments {
                    dead_code_visit_expression(expr, used_symbols, compile_settings);
                }
                used_symbols.poison_properties(compile_settings);
            }
            StatementKind::Spawn {
                arguments: values, ..
            }
            | StatementKind::Return { values } => {
                for expr in values {
                    dead_code_visit_expression(expr, used_symbols, compile_settings);
                }
            }
        }
    }
}

fn dead_code_visit_expression(
    expression: &Expression,
    used_symbols: &mut UsedAssignmentsSet,
    compile_settings: &CompileSettings,
) {
    match &expression.kind {
        ExpressionKind::Integer(_)
        | ExpressionKind::Fix(_)
        | ExpressionKind::Bool(_)
        | ExpressionKind::Error
        | ExpressionKind::Nop => {}
        ExpressionKind::BinaryOperation {
            ref lhs, ref rhs, ..
        } => {
            dead_code_visit_expression(lhs, used_symbols, compile_settings);
            dead_code_visit_expression(rhs, used_symbols, compile_settings);
        }
        ExpressionKind::Call { arguments, .. } => {
            for expression in arguments {
                dead_code_visit_expression(expression, used_symbols, compile_settings);
            }
            used_symbols.poison_properties(compile_settings);
        }
        ExpressionKind::Variable(_) => {
            let symbol = expression.meta.get().unwrap();
            used_symbols.used(*symbol);
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
    fn dead_code_snapshot_tests() {
        glob!("snapshot_tests", "dead_code/*.tapir", |path| {
            let input = fs::read_to_string(path).unwrap();

            let lexer = Lexer::new(&input, FileId::new(0));
            let parser = grammar::ScriptParser::new();
            let file_id = FileId::new(0);

            let mut diagnostics = Diagnostics::new(file_id, path.file_name().unwrap(), &input);

            let mut script = parser.parse(file_id, &mut diagnostics, lexer).unwrap();

            let compile_settings = CompileSettings {
                properties: vec![
                    Property {
                        ty: Type::Int,
                        index: 0,
                        name: "int_prop".to_owned(),
                    },
                    Property {
                        ty: Type::Fix,
                        index: 1,
                        name: "fix_prop".to_owned(),
                    },
                    Property {
                        ty: Type::Bool,
                        index: 2,
                        name: "bool_prop".to_owned(),
                    },
                ],
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

                dead_code_eliminate(function, &compile_settings);
            }

            assert_ron_snapshot!(script, {
                ".**.span" => "[span]",
                ".**.meta" => "[meta]",
            });
        });
    }
}