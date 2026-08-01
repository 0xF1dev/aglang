use crate::error::{SyntaxError, format_error};
use crate::parser::{Argument, Statement, StatementTypes};
use clap::ValueEnum;

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum Target {
    C,
}

pub fn transpile_to_c(statements: Vec<Statement>) -> Result<String, String> {
    let mut code = String::new();

    code.push_str("#include <stdio.h>\n\n");
    if statements
        .iter()
        .any(|s| s.statement_type == StatementTypes::Input)
    {
        code.push_str("#include <string.h>\n\nchar buffer[256] = {0};\n\n");
    }
    code.push_str("unsigned char r0 = 0;\nunsigned char r1 = 0;\n\n");
    code.push_str("unsigned char stack[1024];\nunsigned int sp = 0;\n\n");
    code.push_str("void push(unsigned char c) { stack[sp++] = c; }\nvoid pop() { sp--; }\n\n");
    code.push_str("int main() {\n");

    for (statement_index, statement) in statements.iter().enumerate() {
        match statement.statement_type {
            StatementTypes::LoopStart => {
                code.push_str("  do {\n");
            }
            StatementTypes::LoopEnd => {
                code.push_str("  } while (stack[sp - 1] != 0);\n");
            }
            StatementTypes::Label => {
                let index = match statement.arg2.unwrap() {
                    Argument::Label { index: i } => i,
                    _ => {
                        return Err(format_error(
                            Box::new(SyntaxError::InvalidArguments),
                            statement_index as u32,
                            format!(
                                "Cannot use argument of type {} as label definition.",
                                statement.arg2.unwrap()
                            ),
                        ));
                    }
                };
                code.push_str(format!("label{index}:\n").as_str());
            }
            StatementTypes::Compare => {
                if statements.len() - 1 != statement_index {
                    let next_statement = statements[statement_index + 1];
                    let index = match next_statement.arg2.unwrap() {
                        Argument::Label { index: i } => i,
                        _ => {
                            return Err(format_error(
                                Box::new(SyntaxError::InvalidArguments),
                                statement_index as u32,
                                format!(
                                    "Cannot use argument of type {} as label reference.",
                                    statement.arg2.unwrap()
                                ),
                            ));
                        }
                    };
                    match next_statement.statement_type {
                        StatementTypes::Greater => code.push_str(
                            format!(
                                "  if ({} > {}) {{\n",
                                arg_to_str(statement.arg1.unwrap(), statement_index as u32)?,
                                arg_to_str(statement.arg2.unwrap(), statement_index as u32)?
                            )
                                .as_str(),
                        ),
                        StatementTypes::Less => code.push_str(
                            format!(
                                "  if ({} < {}) {{\n",
                                arg_to_str(statement.arg1.unwrap(), statement_index as u32)?,
                                arg_to_str(statement.arg2.unwrap(), statement_index as u32)?
                            )
                                .as_str(),
                        ),
                        StatementTypes::Equal => code.push_str(
                            format!(
                                "  if ({} == {}) {{\n",
                                arg_to_str(statement.arg1.unwrap(), statement_index as u32)?,
                                arg_to_str(statement.arg2.unwrap(), statement_index as u32)?
                            )
                                .as_str(),
                        ),
                        _ => {
                            return Err(format_error(
                                Box::new(SyntaxError::InvalidStatement),
                                statement_index as u32,
                                format!(
                                    "Statement {:?} has to be preceded by a {:?} statement.",
                                    statement.statement_type,
                                    StatementTypes::Compare
                                ),
                            ));
                        }
                    }
                    code.push_str(format!("    goto label{};\n  }}\n", index).as_str())
                }
            }
            StatementTypes::Greater | StatementTypes::Less | StatementTypes::Equal => continue,
            StatementTypes::Copy => match statement.arg2.unwrap() {
                Argument::Stack => code.push_str(
                    format!(
                        "  push({});\n",
                        arg_to_str(statement.arg1.unwrap(), statement_index as u32)?
                    )
                        .as_str()
                ),
                Argument::Literal(_) => {
                    return Err(format_error(
                        Box::new(SyntaxError::InvalidArguments),
                        statement_index as u32,
                        format!("Invalid arguments supplied for {:?}", StatementTypes::Copy),
                    ));
                }
                Argument::StdOut { as_number: false } => code.push_str(
                    format!(
                        "  putchar({});\n",
                        arg_to_str(statement.arg1.unwrap(), statement_index as u32)?
                    )
                        .as_str()
                ),
                Argument::StdOut { as_number: true } => code.push_str(
                    format!(
                        "  printf(\"%d\", {});\n",
                        arg_to_str(statement.arg1.unwrap(), statement_index as u32)?
                    )
                        .as_str()
                ),
                _ => code.push_str(
                    format!(
                        "  {} = {};\n",
                        arg_to_str(statement.arg2.unwrap(), statement_index as u32)?,
                        arg_to_str(statement.arg1.unwrap(), statement_index as u32)?
                    )
                        .as_str(),
                ),
            },
            StatementTypes::Remove => match statement.arg1.unwrap() {
                Argument::Stack => code.push_str("  if (sp != 0) {\n    pop();\n  } else {\n    return 110;\n  };\n"),
                Argument::R0 | Argument::R1 => code.push_str(
                    format!(
                        "  {} = 0;\n",
                        arg_to_str(statement.arg1.unwrap(), statement_index as u32)?
                    )
                        .as_str(),
                ),
                _ => {
                    return Err(format_error(
                        Box::new(SyntaxError::InvalidArguments),
                        statement_index as u32,
                        format!(
                            "Invalid argument supplied for {:?}",
                            StatementTypes::Remove
                        ),
                    ));
                }
            },
            StatementTypes::Input => {
                code.push_str("  push(0);\n  if (fgets(buffer, sizeof(buffer), stdin) != NULL) {\n    buffer[strcspn(buffer, \"\\r\\n\")] = '\\0';\n    for (int i = strlen(buffer) - 1; i >= 0; i--) {\n      push(buffer[i]);\n    }\n  }\n")
            }
            StatementTypes::Add => {
                if matches!(statement.arg1.as_ref(), Some(Argument::Literal(_))) || matches!(statement.arg1.as_ref(), Some(Argument::Stack)) {
                    return Err(format_error(
                        Box::new(SyntaxError::InvalidArguments),
                        statement_index as u32,
                        format!(
                            "Invalid argument supplied for {:?}",
                            StatementTypes::Add
                        ),
                    ));
                }
                code.push_str(format!("  {} += {};\n", arg_to_str(statement.arg1.unwrap(), statement_index as u32)?, arg_to_str(statement.arg2.unwrap(), statement_index as u32)?).as_str());
            }
            StatementTypes::Subtract => {
                if matches!(statement.arg1.as_ref(), Some(Argument::Literal(_))) || matches!(statement.arg1.as_ref(), Some(Argument::Stack)) {
                    return Err(format_error(
                        Box::new(SyntaxError::InvalidArguments),
                        statement_index as u32,
                        format!(
                            "Invalid argument supplied for {:?}",
                            StatementTypes::Subtract
                        ),
                    ));
                }
                code.push_str(format!("  {} -= {};\n", arg_to_str(statement.arg1.unwrap(), statement_index as u32)?, arg_to_str(statement.arg2.unwrap(), statement_index as u32)?).as_str());
            }
            StatementTypes::Multiply => {
                if matches!(statement.arg1.as_ref(), Some(Argument::Literal(_))) || matches!(statement.arg1.as_ref(), Some(Argument::Stack)) {
                    return Err(format_error(
                        Box::new(SyntaxError::InvalidArguments),
                        statement_index as u32,
                        format!(
                            "Invalid argument supplied for {:?}",
                            StatementTypes::Multiply
                        ),
                    ));
                }
                code.push_str(format!("  {} *= {};\n", arg_to_str(statement.arg1.unwrap(), statement_index as u32)?, arg_to_str(statement.arg2.unwrap(), statement_index as u32)?).as_str());
            }
            StatementTypes::Divide => {
                if matches!(statement.arg1.as_ref(), Some(Argument::Literal(_))) || matches!(statement.arg1.as_ref(), Some(Argument::Stack)) {
                    return Err(format_error(
                        Box::new(SyntaxError::InvalidArguments),
                        statement_index as u32,
                        format!(
                            "Invalid argument supplied for {:?}",
                            StatementTypes::Divide
                        ),
                    ));
                }
                code.push_str(format!("  {} /= {};\n", arg_to_str(statement.arg1.unwrap(), statement_index as u32)?, arg_to_str(statement.arg2.unwrap(), statement_index as u32)?).as_str());
            }
            StatementTypes::Remainder => {
                if matches!(statement.arg1.as_ref(), Some(Argument::Literal(_))) || matches!(statement.arg1.as_ref(), Some(Argument::Stack)) {
                    return Err(format_error(
                        Box::new(SyntaxError::InvalidArguments),
                        statement_index as u32,
                        format!(
                            "Invalid argument supplied for {:?}",
                            StatementTypes::Remainder
                        ),
                    ));
                }
                code.push_str(format!("  {} %= {};\n", arg_to_str(statement.arg1.unwrap(), statement_index as u32)?, arg_to_str(statement.arg2.unwrap(), statement_index as u32)?).as_str());
            }
            _ => return Err(format_error(
                Box::new(SyntaxError::InvalidStatement),
                statement_index as u32,
                "Invalid statement provided.",
            )),
        }
    }

    code.push_str("  return 0;\n}\n");

    Ok(code)
}

fn arg_to_str(argument: Argument, statement_index: u32) -> Result<String, String> {
    match argument {
        Argument::R0 => Ok("r0".to_owned()),
        Argument::R1 => Ok("r1".to_owned()),
        Argument::Stack => Ok("stack[sp - 1]".to_owned()),
        Argument::Literal(val) => Ok(val.to_string()),
        _ => Err(format_error(
            Box::new(SyntaxError::InvalidArguments),
            statement_index,
            format!("Argument {:?} is not readable.", argument),
        )),
    }
}
