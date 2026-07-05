use crate::error::{SyntaxError, error};
use regex_lite::Regex;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StatementTypes {
    None,
    Input,
    Copy,
    Remove,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

const VALID_TOKENS: [char; 17] = [
    ';', '0', '1', '[', ']', '\'', '"', ':', '\\', '|', '>', '!', '+', '-', '*', '/', '%',
];
const STATEMENT_TOKENS: [char; 8] = ['|', '>', '!', '+', '-', '*', '/', '%'];

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum LoopState {
    Start,
    End,
    Both,
}

#[derive(Debug, Copy, Clone)]
pub enum Argument {
    Literal(u8),
    R0,
    R1,
    Stack,
    StdOut,
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Argument({self:?})")
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Statement {
    pub statement_type: StatementTypes,
    pub arg1: Option<Argument>,
    pub arg2: Option<Argument>,
    pub loop_state: Option<LoopState>,
}
impl Statement {
    fn new() -> Self {
        Statement {
            statement_type: StatementTypes::None,
            arg1: None,
            arg2: None,
            loop_state: None,
        }
    }
}

pub fn parse_source(src: String) -> Vec<Statement> {
    let re = Regex::new(r"\$[\s\S]*?\$").unwrap();
    let mut filtered_src = re.replace_all(src.as_str(), "").to_string();

    filtered_src.retain(|c| VALID_TOKENS.contains(&c));

    let raw_statements: Vec<&str> = filtered_src
        .trim()
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut statements: Vec<Statement> = Vec::new();

    for (i, raw_statement) in raw_statements.iter().enumerate() {
        statements.push(parse_statement(raw_statement, i as u32))
    }

    statements
}

fn parse_statement(statement: &str, statement_index: u32) -> Statement {
    let mut statement_struct = Statement::new();

    if statement.starts_with('[') && statement.ends_with(']') {
        statement_struct.loop_state = Some(LoopState::Both)
    } else if statement.starts_with('[') {
        statement_struct.loop_state = Some(LoopState::Start)
    } else if statement.ends_with(']') {
        statement_struct.loop_state = Some(LoopState::End)
    }

    let raw_args: Vec<&str> = statement.split(STATEMENT_TOKENS).collect();

    if raw_args.len() > 2 {
        error(
            Box::new(SyntaxError::ChainedOperations),
            statement_index,
            "Cannot chain multiple operations.",
        );
    }

    let mut args: Vec<Argument> = Vec::new();
    for raw_arg in &raw_args {
        let filtered_arg = raw_arg.trim().replace(['[', ']'], "");
        if !filtered_arg.is_empty() {
            args.push(parse_argument(filtered_arg.as_str(), statement_index).unwrap())
        }
    }

    if statement.contains("|") {
        if args.len() != 0 {
            error(
                Box::new(SyntaxError::InvalidArguments),
                statement_index,
                format!(
                    "{:?} requires 0 arguments, {} supplied",
                    StatementTypes::Input,
                    args.len()
                ),
            );
        }
        statement_struct.statement_type = StatementTypes::Input;
    } else if statement.contains(">") {
        if args.len() != 2 {
            error(
                Box::new(SyntaxError::InvalidArguments),
                statement_index,
                format!(
                    "{:?} requires 2 arguments, {} supplied",
                    StatementTypes::Copy,
                    args.len()
                ),
            );
        }
        statement_struct.statement_type = StatementTypes::Copy;
        statement_struct.arg1 = Some(args[0]);
        statement_struct.arg2 = Some(args[1]);
    } else if statement.contains("!") {
        if args.len() != 1 {
            error(
                Box::new(SyntaxError::InvalidArguments),
                statement_index,
                format!(
                    "{:?} requires 1 argument, {} supplied",
                    StatementTypes::Remove,
                    args.len()
                ),
            );
        }
        statement_struct.statement_type = StatementTypes::Remove;
        statement_struct.arg1 = Some(args[0]);
    } else if statement.contains("+") {
        if args.len() != 2 {
            error(
                Box::new(SyntaxError::InvalidArguments),
                statement_index,
                format!(
                    "{:?} requires 2 arguments, {} supplied",
                    StatementTypes::Add,
                    args.len()
                ),
            );
        }
        statement_struct.statement_type = StatementTypes::Add;
        statement_struct.arg1 = Some(args[0]);
        statement_struct.arg2 = Some(args[1]);
    } else if statement.contains("-") {
        if args.len() != 2 {
            error(
                Box::new(SyntaxError::InvalidArguments),
                statement_index,
                format!(
                    "{:?} requires 2 arguments, {} supplied",
                    StatementTypes::Subtract,
                    args.len()
                ),
            );
        }
        statement_struct.statement_type = StatementTypes::Subtract;
        statement_struct.arg1 = Some(args[0]);
        statement_struct.arg2 = Some(args[1]);
    } else if statement.contains("*") {
        if args.len() != 2 {
            error(
                Box::new(SyntaxError::InvalidArguments),
                statement_index,
                format!(
                    "{:?} requires 2 arguments, {} supplied",
                    StatementTypes::Multiply,
                    args.len()
                ),
            );
        }
        statement_struct.statement_type = StatementTypes::Multiply;
        statement_struct.arg1 = Some(args[0]);
        statement_struct.arg2 = Some(args[1]);
    } else if statement.contains("/") {
        if args.len() != 2 {
            error(
                Box::new(SyntaxError::InvalidArguments),
                statement_index,
                format!(
                    "{:?} requires 2 arguments, {} supplied",
                    StatementTypes::Divide,
                    args.len()
                ),
            );
        }
        statement_struct.statement_type = StatementTypes::Divide;
        statement_struct.arg1 = Some(args[0]);
        statement_struct.arg2 = Some(args[1]);
    } else if statement.contains("%") {
        if args.len() != 2 {
            error(
                Box::new(SyntaxError::InvalidArguments),
                statement_index,
                format!(
                    "{:?} requires 2 arguments, {} supplied",
                    StatementTypes::Remainder,
                    args.len()
                ),
            );
        }
        statement_struct.statement_type = StatementTypes::Remainder;
        statement_struct.arg1 = Some(args[0]);
        statement_struct.arg2 = Some(args[1]);
    }

    statement_struct
}

fn parse_argument(arg: &str, statement_index: u32) -> Option<Argument> {
    match arg {
        "'" => Some(Argument::R0),
        "''" | "\"" => Some(Argument::R1),
        ":" => Some(Argument::Stack),
        "\\" => Some(Argument::StdOut),
        s if s.chars().all(|c| c == '0' || c == '1') => match u8::from_str_radix(s, 2) {
            Ok(num) => Some(Argument::Literal(num)),
            Err(_) => {
                error(
                    Box::new(SyntaxError::InvalidArguments),
                    statement_index,
                    format!("Invalid binary literal: {s}"),
                );
                None
            }
        },
        _ => None,
    }
}
