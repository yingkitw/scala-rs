use std::env;
use std::fs;
use std::process;

fn print_usage() {
    println!("scala-rs - A Scala interpreter written in Rust");
    println!();
    println!("Usage:");
    println!("  scala-rs <file.scala>        Run a Scala source file");
    println!("  scala-rs --repl              Start interactive REPL");
    println!("  scala-rs --check <file>      Type-check only");
    println!("  scala-rs --tokens <file>     Dump tokens");
    println!("  scala-rs --ast <file>        Dump AST");
    println!("  scala-rs --help              Show this help");
    println!("  scala-rs --version           Show version");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("scala-rs: no input files");
        println!("Use --repl for interactive mode or <file.scala> to run a file.");
        process::exit(1);
    }

    match args[1].as_str() {
        "--help" | "-h" => {
            print_usage();
        }
        "--version" | "-v" => {
            println!("scala-rs 0.1.0");
        }
        "--repl" => {
            scala_rs::repl::run_repl();
        }
        "--check" => {
            if args.len() < 3 {
                eprintln!("error: --check requires a file argument");
                process::exit(1);
            }
            let path = &args[2];
            let source = fs::read_to_string(path)
                .unwrap_or_else(|e| {
                    eprintln!("error: cannot read '{}': {}", path, e);
                    process::exit(1);
                });
            if let Err(e) = scala_rs::run_file(&source, true, false, false) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        "--tokens" => {
            if args.len() < 3 {
                eprintln!("error: --tokens requires a file argument");
                process::exit(1);
            }
            let path = &args[2];
            let source = fs::read_to_string(path)
                .unwrap_or_else(|e| {
                    eprintln!("error: cannot read '{}': {}", path, e);
                    process::exit(1);
                });
            if let Err(e) = scala_rs::run_file(&source, false, true, false) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        "--ast" => {
            if args.len() < 3 {
                eprintln!("error: --ast requires a file argument");
                process::exit(1);
            }
            let path = &args[2];
            let source = fs::read_to_string(path)
                .unwrap_or_else(|e| {
                    eprintln!("error: cannot read '{}': {}", path, e);
                    process::exit(1);
                });
            if let Err(e) = scala_rs::run_file(&source, false, false, true) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        arg if arg.starts_with('-') => {
            eprintln!("error: unknown option '{}'", arg);
            eprintln!("Use --help for usage information.");
            process::exit(1);
        }
        file => {
            let source = fs::read_to_string(file)
                .unwrap_or_else(|e| {
                    eprintln!("error: cannot read '{}': {}", file, e);
                    process::exit(1);
                });
            if let Err(e) = scala_rs::run_file(&source, false, false, false) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }
}
