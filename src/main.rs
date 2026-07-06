use std::error::Error;
use clap::{Parser, Subcommand};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use interpreter::Interpreter;
use compiler::Compiler;

mod interpreter;
pub mod error;
pub mod parser;
pub mod compiler;

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
        output: String
    }
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
        },
        Commands::Build { file, output } => {
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
            match write_asm_to_file("temp.s", asm) {
                Ok(()) => println!("\x1b[0;32mWritten assembly file successfully."),
                Err(e) => {
                    eprintln!("\x1b[1;31mCould not write assembly file: {e}");
                    std::process::exit(1);
                }
            };
            match compile_asm("temp.s", output.clone()) {
                Ok(()) => {
                    println!("\x1b[0;32mCompiled assembly file with gcc successfully.")
                }
                Err(e) => {
                    eprintln!("\x1b[1;31mCould not compile assembly file with gcc (is it installed?): {e}");
                }
            }
            fs::remove_file("temp.s").unwrap();
        },
    };
}

fn write_asm_to_file(filename: &str, source: String) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    file.write_all(source.as_bytes())?;
    Ok(())
}

fn compile_asm(asm_file: &str, output: String) -> Result<(), Box<dyn Error>> {
    Command::new("gcc").arg(asm_file).arg("-o").arg(output).output()?;

    Ok(())
}
