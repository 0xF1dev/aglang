use std::cmp::PartialEq;
use std::io;
use std::io::Write;

use crate::error::{RuntimeError, SyntaxError, error};
use crate::parser::{Argument, Statement, StatementTypes};

#[derive(Debug)]
struct Label {
    index: u8,
    instruction: u32,
}

#[derive(PartialEq)]
enum Compare {
    Equal,
    Greater,
    Less,
}

pub struct Interpreter {
    r0: u8,
    r1: u8,
    stack: Vec<u8>,
    instruction_pointer: u32,
    loops: Vec<u32>,
    labels: Vec<Label>,
    current_comparison: Option<Compare>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            r0: 0,
            r1: 0,
            stack: Vec::new(),
            instruction_pointer: 0,
            loops: Vec::new(),
            labels: Vec::new(),
            current_comparison: None,
        }
    }

    fn preprocess(&mut self, statements: &Vec<Statement>) {
        for (i, statement) in statements.iter().enumerate() {
            if statement.statement_type == StatementTypes::Label {
                let index = match statement.arg2.unwrap() {
                    Argument::Label { index: i } => i,
                    _ => {
                        error(
                            Box::new(SyntaxError::InvalidArguments),
                            self.instruction_pointer,
                            format!(
                                "Cannot use argument of type {} as label definition.",
                                statement.arg2.unwrap()
                            ),
                        );
                        return;
                    }
                };
                self.labels.push(Label {
                    index,
                    instruction: i as u32,
                })
            }
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) {
        self.preprocess(&statements);

        while self.instruction_pointer < statements.len() as u32 {
            let statement = statements[self.instruction_pointer as usize];

            if statement.statement_type == StatementTypes::Greater
                || statement.statement_type == StatementTypes::Less
                || statement.statement_type == StatementTypes::Equal
            {
                if statements
                    .get(self.instruction_pointer as usize - 1)
                    .is_none()
                    || statements
                        .get(self.instruction_pointer as usize - 1)
                        .unwrap()
                        .statement_type
                        != StatementTypes::Compare
                {
                    error(
                        Box::new(SyntaxError::InvalidStatement),
                        self.instruction_pointer,
                        format!(
                            "Statement {:?} has to be preceded by a {:?} statement.",
                            statement.statement_type,
                            StatementTypes::Compare
                        ),
                    )
                }
            }

            match statement.statement_type {
                StatementTypes::LoopStart => {
                    if !self.loops.contains(&self.instruction_pointer) {
                        self.loops.push(self.instruction_pointer);
                    }
                }
                StatementTypes::LoopEnd => {
                    if self.stack.last() != Some(&0) || self.stack.len() == 0 {
                        self.instruction_pointer = *self.loops.last().unwrap()
                    } else if *self.stack.last().unwrap() == 0 {
                        self.loops.pop();
                    }
                }
                StatementTypes::Compare => {
                    let first_value = self.get_argument_value(statement.arg1.unwrap());
                    let second_value = self.get_argument_value(statement.arg2.unwrap());
                    if first_value == second_value {
                        self.current_comparison = Some(Compare::Equal)
                    } else if first_value > second_value {
                        self.current_comparison = Some(Compare::Greater)
                    } else if first_value < second_value {
                        self.current_comparison = Some(Compare::Less)
                    }
                }
                StatementTypes::Greater => {
                    let label_index = match statement.arg2.unwrap() {
                        Argument::Label { index: i } => i,
                        a => {
                            error(
                                Box::new(SyntaxError::InvalidArguments),
                                self.instruction_pointer,
                                format!("Cannot use argument {:?} as label reference.", a),
                            );
                            0
                        }
                    };
                    let label = self.labels.iter().find(|l| l.index == label_index);
                    if label.is_none() {
                        error(
                            Box::new(SyntaxError::InvalidArguments),
                            self.instruction_pointer,
                            format!("Label {} isn't defined.", ".".repeat(label_index as usize)),
                        )
                    }
                    if self.current_comparison == Some(Compare::Greater) {
                        self.instruction_pointer = label.unwrap().instruction;
                    }
                }
                StatementTypes::Less => {
                    let label_index = match statement.arg2.unwrap() {
                        Argument::Label { index: i } => i,
                        a => {
                            error(
                                Box::new(SyntaxError::InvalidArguments),
                                self.instruction_pointer,
                                format!("Cannot use argument {:?} as label reference.", a),
                            );
                            0
                        }
                    };
                    let label = self.labels.iter().find(|l| l.index == label_index);
                    if label.is_none() {
                        error(
                            Box::new(SyntaxError::InvalidArguments),
                            self.instruction_pointer,
                            format!("Label {} isn't defined.", ".".repeat(label_index as usize)),
                        )
                    }
                    if self.current_comparison == Some(Compare::Less) {
                        self.instruction_pointer = label.unwrap().instruction;
                    }
                }
                StatementTypes::Equal => {
                    let label_index = match statement.arg2.unwrap() {
                        Argument::Label { index: i } => i,
                        a => {
                            error(
                                Box::new(SyntaxError::InvalidArguments),
                                self.instruction_pointer,
                                format!("Cannot use argument {:?} as label reference.", a),
                            );
                            0
                        }
                    };
                    let label = self.labels.iter().find(|l| l.index == label_index);
                    if label.is_none() {
                        error(
                            Box::new(SyntaxError::InvalidArguments),
                            self.instruction_pointer,
                            format!("Label {} isn't defined.", ".".repeat(label_index as usize)),
                        )
                    }
                    if self.current_comparison == Some(Compare::Equal) {
                        self.instruction_pointer = label.unwrap().instruction;
                    }
                }
                StatementTypes::Copy => {
                    let value = self.get_argument_value(statement.arg1.unwrap());
                    match statement.arg2.unwrap() {
                        Argument::R0 => self.r0 = value,
                        Argument::R1 => self.r1 = value,
                        Argument::Stack => self.stack.push(value),
                        Argument::StdOut { as_number: false } => {
                            print!("{}", char::from(value));
                            io::stdout().flush().unwrap()
                        }
                        Argument::StdOut { as_number: true } => {
                            print!("{value}");
                            io::stdout().flush().unwrap()
                        }
                        _ => {
                            error(
                                Box::new(SyntaxError::InvalidDestination),
                                self.instruction_pointer,
                                format!(
                                    "Cannot write to argument of type: {}",
                                    statement.arg1.unwrap()
                                ),
                            );
                            return;
                        }
                    }
                }
                StatementTypes::Input => {
                    let mut input_str = String::new();
                    io::stdin().read_line(&mut input_str).unwrap();
                    let mut input = input_str.into_bytes();
                    input.reverse();
                    self.stack.push(0);
                    self.stack.append(&mut input);
                }
                StatementTypes::Remove => {
                    match statement.arg1.unwrap() {
                        Argument::R0 => self.r0 = 0,
                        Argument::R1 => self.r1 = 0,
                        Argument::Stack => {
                            self.stack.pop();
                        }
                        _ => {
                            error(
                                Box::new(SyntaxError::InvalidDestination),
                                self.instruction_pointer,
                                format!(
                                    "Cannot write to argument of type: {}",
                                    statement.arg1.unwrap()
                                ),
                            );
                            return;
                        }
                    };
                }
                StatementTypes::Add => {
                    let value_to_add = self.get_argument_value(statement.arg2.unwrap());
                    match statement.arg1.unwrap() {
                        Argument::R0 => self.r0 = self.r0.wrapping_add(value_to_add),
                        Argument::R1 => self.r1 = self.r1.wrapping_add(value_to_add),
                        _ => {
                            error(
                                Box::new(SyntaxError::InvalidDestination),
                                self.instruction_pointer,
                                format!(
                                    "Cannot write to argument of type: {}",
                                    statement.arg1.unwrap()
                                ),
                            );
                            return;
                        }
                    }
                }
                StatementTypes::Subtract => {
                    let value_to_subtract = self.get_argument_value(statement.arg2.unwrap());
                    match statement.arg1.unwrap() {
                        Argument::R0 => self.r0 = self.r0.wrapping_sub(value_to_subtract),
                        Argument::R1 => self.r1 = self.r1.wrapping_sub(value_to_subtract),
                        _ => {
                            error(
                                Box::new(SyntaxError::InvalidDestination),
                                self.instruction_pointer,
                                format!(
                                    "Cannot write to argument of type: {}",
                                    statement.arg1.unwrap()
                                ),
                            );
                            return;
                        }
                    }
                }
                StatementTypes::Multiply => {
                    let factor = self.get_argument_value(statement.arg2.unwrap());
                    match statement.arg1.unwrap() {
                        Argument::R0 => self.r0 = self.r0.wrapping_mul(factor),
                        Argument::R1 => self.r1 = self.r1.wrapping_mul(factor),
                        _ => {
                            error(
                                Box::new(SyntaxError::InvalidDestination),
                                self.instruction_pointer,
                                format!(
                                    "Cannot write to argument of type: {}",
                                    statement.arg1.unwrap()
                                ),
                            );
                            return;
                        }
                    }
                }
                StatementTypes::Divide => {
                    let divisor = self.get_argument_value(statement.arg2.unwrap());
                    match statement.arg1.unwrap() {
                        Argument::R0 => self.r0 = self.r0.wrapping_div(divisor),
                        Argument::R1 => self.r1 = self.r1.wrapping_div(divisor),
                        _ => {
                            error(
                                Box::new(SyntaxError::InvalidDestination),
                                self.instruction_pointer,
                                format!(
                                    "Cannot write to argument of type: {}",
                                    statement.arg1.unwrap()
                                ),
                            );
                            return;
                        }
                    }
                }
                StatementTypes::Remainder => {
                    let divisor = self.get_argument_value(statement.arg2.unwrap());
                    match statement.arg1.unwrap() {
                        Argument::R0 => self.r0 = self.r0.wrapping_rem(divisor),
                        Argument::R1 => self.r1 = self.r1.wrapping_rem(divisor),
                        _ => {
                            error(
                                Box::new(SyntaxError::InvalidDestination),
                                self.instruction_pointer,
                                format!(
                                    "Cannot write to argument of type: {}",
                                    statement.arg1.unwrap()
                                ),
                            );
                            return;
                        }
                    }
                }
                _ if statement.statement_type != StatementTypes::Label => error(
                    Box::new(SyntaxError::InvalidStatement),
                    self.instruction_pointer,
                    "Invalid statement provided.",
                ),
                _ => {}
            }

            self.instruction_pointer += 1;
        }
    }

    fn get_argument_value(&self, argument: Argument) -> u8 {
        match argument {
            Argument::Literal(v) => v,
            Argument::R0 => self.r0,
            Argument::R1 => self.r1,
            Argument::Stack => {
                if self.stack.len() > 0 {
                    *self.stack.last().unwrap()
                } else {
                    error(
                        Box::new(RuntimeError::EmptyStackRead),
                        self.instruction_pointer,
                        "Tried to access stack, but stack is empty.",
                    );
                    0
                }
            }
            _ => {
                error(
                    Box::new(SyntaxError::InvalidSource),
                    self.instruction_pointer,
                    format!("Cannot read from argument of type: {argument}"),
                );
                0
            }
        }
    }
}
