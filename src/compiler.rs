use crate::error::{CompileError, SyntaxError, format_error};
use crate::parser::{Argument, Statement, StatementTypes};
use std::cmp::PartialEq;
use clap::ValueEnum;

#[derive(PartialEq, Clone, Copy, ValueEnum, Debug)]
pub enum Target {
    Linux,
    Windows,
}

pub struct Compiler {
    loop_count: u32,
    active_loops: Vec<u32>,
    input_loop_count: u32,
    active_input_loops: Vec<u32>,
    decimal_print_loop_count: u32,
    pushes: u32,
    pops: u32,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            loop_count: 0,
            active_loops: Vec::new(),
            input_loop_count: 0,
            active_input_loops: Vec::new(),
            decimal_print_loop_count: 0,
            pushes: 0,
            pops: 0,
        }
    }

    pub fn compile_to_asm(&mut self, statements: Vec<Statement>, target: Target) -> Result<String, String> {
        let mut asm = String::from(".intel_syntax noprefix\n");
        if target == Target::Linux {
            asm.push_str(".global _start\n\n")
        } else {
            asm.push_str(".global main\n\n")
        }

        if statements
            .iter()
            .any(|s| s.statement_type == StatementTypes::Input)
            && statements
                .iter()
                .any(|s| s.arg2 == Some(Argument::StdOut { as_number: true }))
        {
            asm.push_str(".section .bss\n.lcomm input_buf, 256\n.lcomm decimal_buf, 3\n\n");
        } else if statements
            .iter()
            .any(|s| s.statement_type == StatementTypes::Input)
        {
            asm.push_str(".section .bss\n.lcomm input_buf, 256\n\n");
        } else if statements
            .iter()
            .any(|s| s.arg2 == Some(Argument::StdOut { as_number: true }))
        {
            asm.push_str(".section .bss\n.lcomm decimal_buf, 3\n\n");
        }

        asm.push_str(".section .text\n");

        if target == Target::Linux {
            asm.push_str("_start:\n")
        } else {
            asm.push_str("main:\n")
        }

        asm.push_str("    push r12\n    xor r12, r12\n    push r13\n    xor r13, r13\n    push rbp\n    mov rbp, rsp\n");

        for (statement_index, statement) in statements.iter().enumerate() {
            match statement.statement_type {
                StatementTypes::LoopStart => {
                    self.loop_count += 1;
                    self.active_loops.push(self.loop_count - 1);
                    asm.push_str(format!("    .l{}:\n", self.loop_count - 1).as_str());
                }
                StatementTypes::LoopEnd => {
                    asm.push_str(format!("    cmp byte ptr [rsp], 0\n    jnz .l{}\n", self.active_loops.last().unwrap()).as_str())
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
                    asm.push_str(format!("    .label{index}:\n").as_str())
                }
                StatementTypes::Compare => match (statement.arg1.unwrap(), statement.arg2.unwrap()) {
                    (Argument::Literal(val0), Argument::Literal(val1)) => {
                        asm.push_str(format!("    mov r14, {val0}\n    cmp r14, {val1}\n").as_str())
                    }
                    (Argument::Literal(val), Argument::R0 | Argument::R1) => {
                        asm.push_str(
                            format!(
                                "    mov r14, {val}\n    cmp r14, {}\n",
                                arg_to_asm_string(statement.arg2.unwrap(), ArgSize::Small)
                            )
                                .as_str(),
                        );
                    }
                    (Argument::R0 | Argument::R1, Argument::Literal(val)) => {
                        asm.push_str(
                            format!(
                                "    cmp {0}, {val}\n",
                                arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Full)
                            )
                                .as_str(),
                        );
                    }
                    (Argument::Literal(val), Argument::Stack) => {
                        asm.push_str(format!("    mov r14, {val}\n    cmp r14, [rsp]\n").as_str());
                    }
                    (Argument::Stack, Argument::Literal(val)) => {
                        asm.push_str(format!("    cmp [rsp], {val}\n").as_str());
                    }
                    (Argument::Stack, Argument::R0 | Argument::R1) => {
                        asm.push_str(format!("    cmp [rsp], {}\n", arg_to_asm_string(statement.arg2.unwrap(), ArgSize::Full)).as_str())
                    }
                    (Argument::R0 | Argument::R1, Argument::Stack) => {
                        asm.push_str(format!("    cmp {0}, [rsp]\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Full)).as_str())
                    }
                    (Argument::Stack, Argument::Stack) => {
                        asm.push_str("    cmp [rsp], [rsp]\n")
                    }
                    (Argument::R0 | Argument::R1, Argument::R0 | Argument::R1) => {
                        asm.push_str(format!("    cmp {}, {}\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Full), arg_to_asm_string(statement.arg2.unwrap(), ArgSize::Full)).as_str())
                    }
                    _ => {
                        return Err(format_error(
                            Box::new(SyntaxError::InvalidArguments),
                            statement_index as u32,
                            format!(
                                "Invalid arguments supplied for {:?}",
                                StatementTypes::Compare
                            ),
                        ))
                    }
                }
                StatementTypes::Greater => {
                    let index = match statement.arg2.unwrap() {
                        Argument::Label { index: i } => i,
                        _ => {
                            return Err(format_error(
                                Box::new(SyntaxError::InvalidArguments),
                                statement_index as u32,
                                format!(
                                    "Cannot use argument of type {} as label reference.",
                                    statement.arg2.unwrap()
                                ),
                            ))
                        }
                    };
                    asm.push_str(format!("    jg .label{index}\n").as_str())
                }
                StatementTypes::Less => {
                    let index = match statement.arg2.unwrap() {
                        Argument::Label { index: i } => i,
                        _ => {
                            return Err(format_error(
                                Box::new(SyntaxError::InvalidArguments),
                                statement_index as u32,
                                format!(
                                    "Cannot use argument of type {} as label reference.",
                                    statement.arg2.unwrap()
                                ),
                            ))
                        }
                    };
                    asm.push_str(format!("    jl .label{index}\n").as_str())
                }
                StatementTypes::Equal => {
                    let index = match statement.arg2.unwrap() {
                        Argument::Label { index: i } => i,
                        _ => {
                            return Err(format_error(
                                Box::new(SyntaxError::InvalidArguments),
                                statement_index as u32,
                                format!(
                                    "Cannot use argument of type {} as label reference.",
                                    statement.arg2.unwrap()
                                ),
                            ))
                        }
                    };
                    asm.push_str(format!("    je .label{index}\n").as_str())
                }
                StatementTypes::Copy => match (statement.arg1.unwrap(), statement.arg2.unwrap()) {
                    (Argument::Literal(val), Argument::R0 | Argument::R1) => {
                        asm.push_str(
                            format!(
                                "    mov {}, {val}\n",
                                arg_to_asm_string(statement.arg2.unwrap(), ArgSize::Small)
                            )
                                .as_str(),
                        );
                    }
                    (Argument::Literal(val), Argument::Stack) => {
                        asm.push_str(format!("    push {val}\n").as_str());
                        self.pushes += 1;
                    }
                    (Argument::Literal(val), Argument::StdOut { as_number: false | true }) => {
                        if statement.arg2.unwrap() == (Argument::StdOut { as_number: true }) {
                            if target == Target::Linux {
                                asm.push_str(format!("    mov rax, {val}\n    lea rcx, [decimal_buf + 2 + rip]\n    mov rbx, 10\n    .dec_loop{0}:\n    xor rdx, rdx\n    div rbx\n    add dl, '0'\n    mov [rcx], dl\n    dec rcx\n    test rax, rax\n    jnz .dec_loop{0}\n    lea rdx, [decimal_buf + 2 + rip]\n    sub rdx, rcx\n    inc rcx\n    mov rsi, rcx\n    mov rax, 1\n    mov rdi, 1\n    mov rsi, rcx\n    syscall\n", self.decimal_print_loop_count).as_str());
                            } else {
                                asm.push_str(format!("    mov rax, {val}\n    lea rcx, [decimal_buf + 2 + rip]\n    mov r8, 10\n    .dec_loop{0}:\n    xor rdx, rdx\n    div r8\n    add dl, '0'\n    mov [rcx], dl\n    dec rcx\n    test rax, rax\n    jnz .dec_loop{0}\n    lea rdx, [decimal_buf + 2 + rip]\n    sub rdx, rcx\n    inc rcx\n    mov r14, rcx\n    .print_loop{0}:\n    movzx rcx, byte ptr [r14]\n    cmp rcx, 0\n    je .print_end{0}\n    mov rbp, rsp\n    and rsp, -16\n    sub rsp, 32\n    call putchar\n    mov rsp, rbp\n    inc r14\n    jmp .print_loop{0}\n    .print_end{0}:\n", self.decimal_print_loop_count).as_str())
                            }
                            self.decimal_print_loop_count += 1;
                        } else {
                            if target == Target::Linux {
                                asm.push_str(format!("    push {val}\n    mov rax, 1\n    mov rdi, 1\n    mov rsi, rsp\n    mov rdx, 1\n    syscall\n    pop rax\n").as_str())
                            } else {
                                asm.push_str(format!("    mov rcx, {val}\n    mov rbp, rsp\n    and rsp, -16\n    sub rsp, 32\n    call putchar\n    mov rsp, rbp\n").as_str())
                            }
                        }
                    }
                    (Argument::R0 | Argument::R1, Argument::StdOut { as_number: false | true }) => {
                        if statement.arg2.unwrap() == (Argument::StdOut { as_number: true }) {
                            if target == Target::Linux {
                                asm.push_str(format!("    mov rax, {0}\n    lea rcx, [decimal_buf + 2 + rip]\n    mov rbx, 10\n    .dec_loop{1}:\n    xor rdx, rdx\n    div rbx\n    add dl, '0'\n    mov [rcx], dl\n    dec rcx\n    test rax, rax\n    jnz .dec_loop{1}\n    lea rdx, [decimal_buf + 2 + rip]\n    sub rdx, rcx\n    inc rcx\n    mov rsi, rcx\n    mov rax, 1\n    mov rdi, 1\n    mov rsi, rcx\n    syscall\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Full), self.decimal_print_loop_count).as_str());
                            } else {
                                asm.push_str(format!("    mov rax, {0}\n    lea rcx, [decimal_buf + 2 + rip]\n    mov r8, 10\n    .dec_loop{1}:\n    xor rdx, rdx\n    div r8\n    add dl, '0'\n    mov [rcx], dl\n    dec rcx\n    test rax, rax\n    jnz .dec_loop{1}\n    lea rdx, [decimal_buf + 2 + rip]\n    sub rdx, rcx\n    inc rcx\n    mov r14, rcx\n    .print_loop{1}:\n    movzx rcx, byte ptr [r14]\n    cmp rcx, 0\n    je .print_end{1}\n    mov rbp, rsp\n    and rsp, -16\n    sub rsp, 32\n    call putchar\n    mov rsp, rbp\n    inc r14\n    jmp .print_loop{1}\n    .print_end{1}:\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Full), self.decimal_print_loop_count).as_str())
                            }
                            self.decimal_print_loop_count += 1;
                        } else {
                            if target == Target::Linux {
                                asm.push_str(format!("    push {0}\n    mov rax, 1\n    mov rdi, 1\n    mov rsi, rsp\n    mov rdx, 1\n    syscall\n    pop {0}\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Full)).as_str())
                            } else {
                                asm.push_str(format!("    push {0}\n    pop rcx\n    mov rbp, rsp\n    and rsp, -16\n    sub rsp, 32\n    call putchar\n    mov rsp, rbp\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Full)).as_str())
                            }
                        }
                    }
                    (Argument::Stack, Argument::StdOut { as_number: false | true }) => {
                        if statement.arg2.unwrap() == (Argument::StdOut { as_number: true }) {
                            if target == Target::Linux {
                                asm.push_str(format!("    mov rax, [rsp]\n    lea rcx, [decimal_buf + 2 + rip]\n    mov rbx, 10\n    .dec_loop{0}:\n    xor rdx, rdx\n    div rbx\n    add dl, '0'\n    mov [rcx], dl\n    dec rcx\n    test rax, rax\n    jnz .dec_loop{0}\n    lea rdx, [decimal_buf + 2 + rip]\n    sub rdx, rcx\n    inc rcx\n    mov rsi, rcx\n    mov rax, 1\n    mov rdi, 1\n    mov rsi, rcx\n    syscall\n", self.decimal_print_loop_count).as_str());
                            } else {
                                asm.push_str(format!("    mov rax, [rsp]\n    lea rcx, [decimal_buf + 2 + rip]\n    mov r8, 10\n    .dec_loop{0}:\n    xor rdx, rdx\n    div r8\n    add dl, '0'\n    mov [rcx], dl\n    dec rcx\n    test rax, rax\n    jnz .dec_loop{0}\n    lea rdx, [decimal_buf + 2 + rip]\n    sub rdx, rcx\n    inc rcx\n    mov r14, rcx\n    .print_loop{0}:\n    movzx rcx, byte ptr [r14]\n    cmp rcx, 0\n    je .print_end{0}\n    mov rbp, rsp\n    and rsp, -16\n    sub rsp, 32\n    call putchar\n    mov rsp, rbp\n    inc r14\n    jmp .print_loop{0}\n    .print_end{0}:\n", self.decimal_print_loop_count).as_str())
                            }
                            self.decimal_print_loop_count += 1;
                        } else {
                            if target == Target::Linux {
                                asm.push_str("    mov rax, 1\n    mov rdi, 1\n    mov rsi, rsp\n    mov rdx, 1\n    syscall\n")
                            } else {
                                asm.push_str("    mov rcx, [rsp]\n    mov rbp, rsp\n    and rsp, -16\n    sub rsp, 32\n    call putchar\n    mov rsp, rbp\n")
                            }
                        }
                    }
                    (Argument::Stack, Argument::R0 | Argument::R1) => {
                        asm.push_str(format!("    mov {0}, [rsp]\n", arg_to_asm_string(statement.arg2.unwrap(), ArgSize::Full)).as_str())
                    }
                    (Argument::R0 | Argument::R1, Argument::Stack) => {
                        asm.push_str(format!("    push {}\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Full)).as_str());
                        self.pushes += 1
                    }
                    _ => {
                        return Err(format_error(
                            Box::new(SyntaxError::InvalidArguments),
                            statement_index as u32,
                            format!(
                                "Invalid arguments supplied for {:?}",
                                StatementTypes::Copy
                            ),
                        ));
                    }
                },
                StatementTypes::Remove => match statement.arg1.unwrap() {
                    Argument::R0 | Argument::R1 => {
                        asm.push_str(format!("    xor {0}, {0}\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Full)).as_str())
                    }
                    Argument::Stack => {
                        asm.push_str("    pop rax\n");
                        self.pops += 1;
                        if self.pops > self.pushes {
                            return Err(format_error(Box::new(CompileError::StackUnderflow), statement_index as u32, "Popping would underflow the stack."));
                        }
                    }
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
                    self.input_loop_count += 1;
                    self.active_input_loops.push(self.input_loop_count - 1);
                    asm.push_str(format!("    mov rax, 0\n    mov rdi, 0\n    lea rsi, [rip + input_buf]\n    mov rdx, 256\n    syscall\n    mov rcx, rax\n    sub rcx, 2\n    push 0\n.input_loop{0}:\n    lea rbx, [rip + input_buf]\n    movzx rbx, byte ptr [rbx + rcx]\n    push rbx\n    dec rcx\n    cmp rcx, 0\n    jge .input_loop{0}\n", self.input_loop_count - 1).as_str())
                }
                StatementTypes::Add => match (statement.arg1.unwrap(), statement.arg2.unwrap()) {
                    (Argument::R0 | Argument::R1, Argument::Literal(val)) => {
                        asm.push_str(format!("    add {}, {val}\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    (Argument::R0 | Argument::R1, Argument::R0 | Argument::R1) => {
                        asm.push_str(format!("    add {}, {}\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small), arg_to_asm_string(statement.arg2.unwrap(), ArgSize::Small)).as_str())
                    }
                    (Argument::R0 | Argument::R1, Argument::Stack) => {
                        asm.push_str(format!("    add {}, [rsp]\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    _ => {
                        return Err(format_error(
                            Box::new(SyntaxError::InvalidArguments),
                            statement_index as u32,
                            format!(
                                "Invalid arguments supplied for {:?}",
                                StatementTypes::Add
                            ),
                        ));
                    }
                },
                StatementTypes::Subtract => match (statement.arg1.unwrap(), statement.arg2.unwrap()) {
                    (Argument::R0 | Argument::R1, Argument::Literal(val)) => {
                        asm.push_str(format!("    sub {}, {val}\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    (Argument::R0 | Argument::R1, Argument::R0 | Argument::R1) => {
                        asm.push_str(format!("    sub {}, {}\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small), arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    (Argument::R0 | Argument::R1, Argument::Stack) => {
                        asm.push_str(format!("    sub {}, [rsp]\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    _ => {
                        return Err(format_error(
                            Box::new(SyntaxError::InvalidArguments),
                            statement_index as u32,
                            format!(
                                "Invalid arguments supplied for {:?}",
                                StatementTypes::Subtract
                            ),
                        ));
                    }
                },
                StatementTypes::Multiply => match (statement.arg1.unwrap(), statement.arg2.unwrap()) {
                    (Argument::R0 | Argument::R1, Argument::Literal(val)) => {
                        asm.push_str(format!("    mul {}, {val}\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    (Argument::R0 | Argument::R1, Argument::R0 | Argument::R1) => {
                        asm.push_str(format!("    mul {}, {}\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small), arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    (Argument::R0 | Argument::R1, Argument::Stack) => {
                        asm.push_str(format!("    mul {}, [rsp]\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    _ => {
                        return Err(format_error(
                            Box::new(SyntaxError::InvalidArguments),
                            statement_index as u32,
                            format!(
                                "Invalid arguments supplied for {:?}",
                                StatementTypes::Multiply
                            ),
                        ));
                    }
                },
                StatementTypes::Divide => match (statement.arg1.unwrap(), statement.arg2.unwrap()) {
                    (Argument::R0 | Argument::R1, Argument::Literal(val)) => {
                        asm.push_str(format!("    mov al, {0}\n    mov ah, 0\n    mov r14b, {val}\n    div r14b\n    mov {0}, al\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    (Argument::R0 | Argument::R1, Argument::R0 | Argument::R1) => {
                        asm.push_str(format!("    mov al, {0}\n    mov ah, 0\n    mov r14b, {1}\n    div r14b\n    mov {0}, al\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small), arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    (Argument::R0 | Argument::R1, Argument::Stack) => {
                        asm.push_str(format!("    mov al, {0}\n    mov ah, 0\n    mov r14b, [rsp]\n    div r14b\n    mov {0}, al\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    _ => {
                        return Err(format_error(
                            Box::new(SyntaxError::InvalidArguments),
                            statement_index as u32,
                            format!(
                                "Invalid arguments supplied for {:?}",
                                StatementTypes::Divide
                            ),
                        ));
                    }
                },
                StatementTypes::Remainder => match (statement.arg1.unwrap(), statement.arg2.unwrap()) {
                    (Argument::R0 | Argument::R1, Argument::Literal(val)) => {
                        asm.push_str(format!("    mov al, {0}\n    mov ah, 0\n    mov r14b, {val}\n    div r14b\n    mov {0}, ah\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    (Argument::R0 | Argument::R1, Argument::R0 | Argument::R1) => {
                        asm.push_str(format!("    mov al, {0}\n    mov ah, 0\n    mov r14b, {1}\n    div r14b\n    mov {0}, ah\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small), arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    (Argument::R0 | Argument::R1, Argument::Stack) => {
                        asm.push_str(format!("    mov al, {0}\n    mov ah, 0\n    mov r14b, [rsp]\n    div r14b\n    mov {0}, ah\n", arg_to_asm_string(statement.arg1.unwrap(), ArgSize::Small)).as_str())
                    }
                    _ => {
                        return Err(format_error(
                            Box::new(SyntaxError::InvalidArguments),
                            statement_index as u32,
                            format!(
                                "Invalid arguments supplied for {:?}",
                                StatementTypes::Remainder
                            ),
                        ));
                    }
                }
                _ => return Err(format_error(
                    Box::new(SyntaxError::InvalidStatement),
                    statement_index as u32,
                    "Invalid statement provided.",
                )),
            }
        }

        asm.push_str("    mov rsp, rbp\n    pop rbp\n    pop r13\n    pop r12\n");

        if target == Target::Linux {
            asm.push_str("    mov rax, 60\n    mov rdi, 0\n    syscall\n")
        } else {
            asm.push_str("    and rsp, -16\n    sub rsp, 32\n    mov rcx, 0\n    call exit\n    add rsp, 32\n")
        }

        Ok(asm)
    }
}

enum ArgSize {
    Full,
    Small,
}

fn arg_to_asm_string(argument: Argument, role: ArgSize) -> String {
    match (argument, role) {
        (Argument::R0, ArgSize::Small) => "r12b".to_owned(),
        (Argument::R0, ArgSize::Full) => "r12".to_owned(),
        (Argument::R1, ArgSize::Small) => "r13b".to_owned(),
        (Argument::R1, ArgSize::Full) => "r13".to_owned(),
        _ => "".to_owned(),
    }
}
