use clap::{Parser, Subcommand};
use compiler::Compiler;
use interpreter::Interpreter;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::ErrorKind::NotFound;
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub mod compiler;
pub mod error;
mod interpreter;
pub mod parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// A simple interpreted and compiled esoteric language.
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an Aglang file with the interpreter
    Run {
        /// Aglang source file
        file: String,
    },

    /// Compile an Aglang file into a Linux ELF64 executable
    Build {
        /// Aglang source file
        file: String,

        /// Output file
        #[arg(short, long)]
        output: String,

        /// Keep the output assembly instead of deleting it
        #[arg(long)]
        keep_asm: bool,

        /// Keep the output object file instead of deleting it
        #[arg(long)]
        keep_obj: bool,
    },
}

#[derive(Debug)]
enum CompilePhase {
    Assembler,
    Linker
}

struct CompileError {
    phase: CompilePhase,
    code: i32,
    output: Vec<u8>,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { file } => {
            let src = match fs::read_to_string(file) {
                Ok(src) => src,
                Err(e) => {
                    eprintln!("\x1b[1;31mCould not open source file: {e}");
                    std::process::exit(1);
                }
            };
            let statements = parser::parse_source(src);
            let mut ip = Interpreter::new();
            ip.interpret(statements);
        }
        Commands::Build {
            file,
            output,
            keep_asm,
            keep_obj
        } => {
            if std::env::consts::OS != "linux" || std::env::consts::ARCH != "x86_64" {
                eprintln!(
                    "\x1b[1;31mCurrently, the compiler only supports x86-64 Linux, but the detected setup is {} {}",
                    std::env::consts::ARCH,
                    std::env::consts::OS
                );
                std::process::exit(1);
            }
            let src = match fs::read_to_string(file) {
                Ok(src) => src,
                Err(e) => {
                    eprintln!("\x1b[1;31mCould not open source file: {e}");
                    std::process::exit(1);
                }
            };
            let statements = parser::parse_source(src);
            let mut compiler = Compiler::new();
            let asm = compiler.compile_to_asm(statements);
            let basename = Path::new(output)
                .file_stem()
                .unwrap_or(OsStr::new("aglang_program"))
                .to_str()
                .unwrap();
            match write_asm_to_file(format!("{basename}.s"), asm) {
                Ok(()) => println!("\x1b[0;32mAssembly file written."),
                Err(e) => {
                    eprintln!("\x1b[1;31mCould not write assembly file: {e}");
                    std::process::exit(1);
                }
            };
            match compile_asm(
                format!("{basename}.s"),
                format!("{basename}.o"),
                output.clone(),
            ) {
                Ok(()) => {
                    println!("\x1b[0;32mProgram compiled successfully.")
                }
                Err(e) => {
                    eprintln!(
                        "\x1b[1;31mCould not compile assembly file (phase: {:?}, error code: {}): {}",
                        e.phase,
                        e.code,
                        String::from_utf8(e.output)
                            .unwrap_or("INVALID UTF8 OUTPUT FROM BUILD COMMAND".to_string())
                    );
                }
            }
            if !keep_asm {
                fs::remove_file(format!("{basename}.s")).unwrap();
            }
            if !keep_obj {
                fs::remove_file(format!("{basename}.o")).unwrap();
            }
        }
    };
}

fn write_asm_to_file(filename: String, source: String) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    file.write_all(source.as_bytes())?;
    Ok(())
}

fn compile_asm(asm_file: String, obj_file: String, output: String) -> Result<(), CompileError> {
    let cmd_output = match Command::new("as")
        .arg(asm_file)
        .arg("-o")
        .arg(obj_file.clone())
        .output() {
        Ok(o) => o,
        Err(e) if e.kind() == NotFound => {
            eprintln!("\x1b[1;31mCommand \"as\" not found. Is GCC installed?");
            std::process::exit(1)
        }
        Err(e) => {
            eprintln!("\x1b[1;31mCould not run \"as\" command: {e}");
            std::process::exit(1)
        }
    };

    if !cmd_output.status.success() {
        return Err(CompileError{ phase: CompilePhase::Assembler, code: cmd_output.status.code().unwrap_or(1), output: cmd_output.stderr })
    }

    let cmd_output = match Command::new("ld")
        .arg("-s")
        .arg("-n")
        .arg(obj_file.clone())
        .arg("-o")
        .arg(output)
        .output() {
        Ok(o) => o,
        Err(e) if e.kind() == NotFound => {
            eprintln!("\x1b[1;31mCommand \"ld\" not found. Is it installed?");
            std::process::exit(1)
        }
        Err(e) => {
            eprintln!("\x1b[1;31mCould not run \"ld\" command: {e}");
            std::process::exit(1)
        }
    };

    if !cmd_output.status.success() {
        return Err(CompileError{ phase: CompilePhase::Linker, code: cmd_output.status.code().unwrap_or(1), output: cmd_output.stderr })
    }

    Ok(())
}
