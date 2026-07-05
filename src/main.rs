use clap::{Parser, Subcommand};
use std::fs;
use interpreter::Interpreter;

mod interpreter;
pub mod error;
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
                    eprintln!("Could not open source file: {e}");
                    std::process::exit(1);
                }
            };
            let statements = parser::parse_source(src);
            let mut ip = Interpreter::new();
            ip.interpret(statements);
        },
        _ => {}
    };
}
