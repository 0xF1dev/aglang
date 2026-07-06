use std::io;
use std::io::Write;

use crate::parser::{Statement, StatementTypes, Argument, LoopState};
use crate::error::{SyntaxError, error, RuntimeError};

pub struct Interpreter {
    r0: u8,
    r1: u8,
    stack: Vec<u8>,
    instruction_pointer: u32,
    loops: Vec<u32>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            r0: 0,
            r1: 0,
            stack: Vec::new(),
            instruction_pointer: 0,
            loops: Vec::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) {
        while self.instruction_pointer < statements.len() as u32 {
            let statement = statements[self.instruction_pointer as usize];

            if (statement.loop_state == Some(LoopState::Start)
                || statement.loop_state == Some(LoopState::Both))
                && !self.loops.contains(&self.instruction_pointer)
            {
                self.loops.push(self.instruction_pointer);
            }

            match statement.statement_type {
                StatementTypes::Copy => {
                    let value = self.get_argument_value(statement.arg1.unwrap());
                    match statement.arg2.unwrap() {
                        Argument::R0 => self.r0 = value,
                        Argument::R1 => self.r1 = value,
                        Argument::Stack => self.stack.push(value),
                        Argument::StdOut => { print!("{}", char::from(value)); io::stdout().flush().unwrap() },
                        _ => {
                            error(
                                Box::new(SyntaxError::InvalidDestination),
                                self.instruction_pointer,
                                format!("Cannot write to argument of type: {}", statement.arg1.unwrap()),
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
                                format!("Cannot write to argument of type: {}", statement.arg1.unwrap()),
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
                                format!("Cannot write to argument of type: {}", statement.arg1.unwrap()),
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
                                format!("Cannot write to argument of type: {}", statement.arg1.unwrap()),
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
                                format!("Cannot write to argument of type: {}", statement.arg1.unwrap()),
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
                                format!("Cannot write to argument of type: {}", statement.arg1.unwrap()),
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
                                format!("Cannot write to argument of type: {}", statement.arg1.unwrap()),
                            );
                            return;
                        }
                    }
                }
                StatementTypes::None
                    if statement.loop_state == Some(LoopState::End)
                        || statement.loop_state == Some(LoopState::Start)
                        || statement.loop_state == Some(LoopState::Both) => {}
                _ => error(
                    Box::new(SyntaxError::InvalidStatement),
                    self.instruction_pointer,
                    "Invalid statement provided.",
                ),
            }

            if (statement.loop_state == Some(LoopState::End)
                || statement.loop_state == Some(LoopState::Both))
                && (self.stack.last() != Some(&0) || self.stack.len() == 0)
            {
                self.instruction_pointer = *self.loops.last().unwrap()
            } else if statement.loop_state == Some(LoopState::End)
                || statement.loop_state == Some(LoopState::Both) && *self.stack.last().unwrap() == 0
            {
                self.loops.pop();
                self.instruction_pointer += 1;
            } else {
                self.instruction_pointer += 1;
            }
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
