use clap::{Parser, Subcommand};
use compiler::Target;
use interpreter::Interpreter;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::ErrorKind::NotFound;
use std::io::Write;
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

        /// Target platform
        #[arg(short, long, value_enum)]
        target: Option<Target>,

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
    Linker,
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
            target,
            keep_asm,
            keep_obj,
        } => {
            if (std::env::consts::OS != "linux" || std::env::consts::ARCH != "x86_64")
                && (std::env::consts::OS != "windows" || std::env::consts::ARCH != "x86_64")
            {
                eprintln!(
                    "\x1b[1;31mCurrently, the compiler only supports x86-64 Linux and x86-64 Windows, but the detected setup is {} {}",
                    std::env::consts::ARCH,
                    std::env::consts::OS
                );
                std::process::exit(1);
            }
            let target = match target {
                Some(t) => *t,
                None => {
                    if std::env::consts::OS == "linux" {
                        Target::Linux
                    } else if std::env::consts::OS == "windows" {
                        Target::Windows
                    } else {
                        eprintln!(
                            "\x1b[1;31mCurrently, the compiler only supports x86-64 Linux and x86-64 Windows, but the detected setup is {} {}",
                            std::env::consts::ARCH,
                            std::env::consts::OS
                        );
                        std::process::exit(1);
                    }
                }
            };
            let keep_asm_str = if *keep_asm { "Yes" } else { "No" };
            let keep_obj_str = if *keep_obj { "Yes" } else { "No" };
            println!("\x1b[1mBuild info:\x1b[0m\n    - Target platform: {target:?} (x86_64)\n    - Keep ASM: {keep_asm_str} ({output}.s)\n    - Keep OBJ: {keep_obj_str} ({output}.o)");
            let src = match fs::read_to_string(file) {
                Ok(src) => src,
                Err(e) => {
                    eprintln!("\x1b[1;31mCould not open source file: {e}");
                    std::process::exit(1);
                }
            };
            let mut spinner =
                Spinner::with_timer(Spinners::Dots, "Compiling code into assembly...".into());
            let statements = parser::parse_source(src);
            let asm = match compiler::compile_to_asm(statements, target) {
                Ok(s) => s,
                Err(e) => {
                    spinner.stop_and_persist(
                        "\x1b[1;31m✖",
                        format!("Could not generate assembly: \x1b[0m {e}"),
                    );
                    std::process::exit(1);
                }
            };
            match write_asm_to_file(format!("{output}.s"), asm) {
                Ok(()) => {
                    spinner.stop_and_persist("\x1b[0;32m🗸", "Assembly generated!\x1b[0m".into())
                }
                Err(e) => {
                    spinner.stop_and_persist(
                        "\x1b[1;31m✖",
                        format!("Could not write assembly: \x1b[0m {e}"),
                    );
                    std::process::exit(1);
                }
            };
            let mut spinner = Spinner::with_timer(Spinners::Dots, "Compiling assembly into executable...".into());
            match compile_asm(
                format!("{output}.s"),
                format!("{output}.o"),
                output.clone(),
                target,
            ) {
                Ok(()) => {
                    spinner.stop_and_persist(
                        "\x1b[0;32m🗸",
                        "Program compiler successfully!\x1b[0m".into(),
                    );
                }
                Err(e) => {
                    spinner.stop_and_persist(
                        "\x1b[1;31m✖",
                        format!(
                            "Could not compile assembly file (phase: {:?}, error code: {}): {}",
                            e.phase,
                            e.code,
                            String::from_utf8(e.output)
                                .unwrap_or("INVALID UTF8 OUTPUT FROM BUILD COMMAND".to_string())
                        ),
                    );
                    std::process::exit(1);
                }
            }
            if !keep_asm {
                let mut spinner = Spinner::with_timer(Spinners::Dots, "Removing assembly file...".into());
                fs::remove_file(format!("{output}.s")).unwrap();
                spinner.stop_and_persist(
                    "\x1b[0;32m🗸",
                    "Assembly file removed!\x1b[0m".into(),
                );
            }
            if !keep_obj {
                let mut spinner = Spinner::with_timer(Spinners::Dots, "Removing object file...".into());
                fs::remove_file(format!("{output}.o")).unwrap();
                spinner.stop_and_persist(
                    "\x1b[0;32m🗸",
                    "Object file removed!\x1b[0m".into(),
                );
            }
            println!("\x1b[1;92mBuild completed!\n\x1b[0;mThe output executable is in \x1b[1m{output}")
        }
    };
}

fn write_asm_to_file(filename: String, source: String) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    file.write_all(source.as_bytes())?;
    Ok(())
}

fn compile_asm(
    asm_file: String,
    obj_file: String,
    output: String,
    target: Target,
) -> Result<(), CompileError> {
    if std::env::consts::OS == "linux" {
        match target {
            Target::Linux => {
                let cmd_output = match Command::new("as")
                    .arg(asm_file)
                    .arg("-o")
                    .arg(obj_file.clone())
                    .output()
                {
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
                    return Err(CompileError {
                        phase: CompilePhase::Assembler,
                        code: cmd_output.status.code().unwrap_or(1),
                        output: cmd_output.stderr,
                    });
                }

                let cmd_output = match Command::new("ld")
                    .arg("-s")
                    .arg("-n")
                    .arg(obj_file.clone())
                    .arg("-o")
                    .arg(output)
                    .output()
                {
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
                    return Err(CompileError {
                        phase: CompilePhase::Linker,
                        code: cmd_output.status.code().unwrap_or(1),
                        output: cmd_output.stderr,
                    });
                }
            }
            Target::Windows => {
                let cmd_output = match Command::new("x86_64-w64-mingw32-as")
                    .arg(asm_file)
                    .arg("-o")
                    .arg(obj_file.clone())
                    .output()
                {
                    Ok(o) => o,
                    Err(e) if e.kind() == NotFound => {
                        eprintln!(
                            "\x1b[1;31mCommand \"x86_64-w64-mingw32-as\" not found. Is MinGW-W64 installed?"
                        );
                        std::process::exit(1)
                    }
                    Err(e) => {
                        eprintln!("\x1b[1;31mCould not run \"x86_64-w64-mingw32-as\" command: {e}");
                        std::process::exit(1)
                    }
                };

                if !cmd_output.status.success() {
                    return Err(CompileError {
                        phase: CompilePhase::Assembler,
                        code: cmd_output.status.code().unwrap_or(1),
                        output: cmd_output.stderr,
                    });
                }

                let cmd_output = match Command::new("x86_64-w64-mingw32-ld")
                    .arg("-s")
                    .arg("-n")
                    .arg(obj_file.clone())
                    .arg("-lmsvcrt")
                    .arg("-o")
                    .arg(output)
                    .output()
                {
                    Ok(o) => o,
                    Err(e) if e.kind() == NotFound => {
                        eprintln!(
                            "\x1b[1;31mCommand \"x86_64-w64-mingw32-ld\" not found. Is MinGW-W64 installed?"
                        );
                        std::process::exit(1)
                    }
                    Err(e) => {
                        eprintln!("\x1b[1;31mCould not run \"x86_64-w64-mingw32-ld\" command: {e}");
                        std::process::exit(1)
                    }
                };

                if !cmd_output.status.success() {
                    return Err(CompileError {
                        phase: CompilePhase::Linker,
                        code: cmd_output.status.code().unwrap_or(1),
                        output: cmd_output.stderr,
                    });
                }
            }
        }
    } else if std::env::consts::OS == "windows" {
        match target {
            Target::Windows => {
                let cmd_output = match Command::new("as.exe")
                    .arg(asm_file)
                    .arg("-o")
                    .arg(obj_file.clone())
                    .output()
                {
                    Ok(o) => o,
                    Err(e) if e.kind() == NotFound => {
                        eprintln!("\x1b[1;31mCommand \"as.exe\" not found. Is MinGW installed?");
                        std::process::exit(1)
                    }
                    Err(e) => {
                        eprintln!("\x1b[1;31mCould not run \"as.exe\" command: {e}");
                        std::process::exit(1)
                    }
                };

                if !cmd_output.status.success() {
                    return Err(CompileError {
                        phase: CompilePhase::Assembler,
                        code: cmd_output.status.code().unwrap_or(1),
                        output: cmd_output.stderr,
                    });
                }

                let cmd_output = match Command::new("ld.exe")
                    .arg("-s")
                    .arg("-n")
                    .arg(obj_file.clone())
                    .arg("-lmsvcrt")
                    .arg("-o")
                    .arg(output)
                    .output()
                {
                    Ok(o) => o,
                    Err(e) if e.kind() == NotFound => {
                        eprintln!("\x1b[1;31mCommand \"ld.exe\" not found. Is MinGW installed?");
                        std::process::exit(1)
                    }
                    Err(e) => {
                        eprintln!("\x1b[1;31mCould not run \"ld.exe\" command: {e}");
                        std::process::exit(1)
                    }
                };

                if !cmd_output.status.success() {
                    return Err(CompileError {
                        phase: CompilePhase::Linker,
                        code: cmd_output.status.code().unwrap_or(1),
                        output: cmd_output.stderr,
                    });
                }
            }
            Target::Linux => {
                eprintln!("\x1b[1;31mCross-compiling from Windows to Linux is not supported.");
                std::process::exit(1)
            }
        }
    }

    Ok(())
}
