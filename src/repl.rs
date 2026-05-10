use crate::interpreter::Interpreter;
use crate::value::Value;

pub fn run_repl() {
    println!("scala-rs REPL (type :quit to exit, :help for help)");

    let mut interp = Interpreter::new();
    let rl = rustyline::DefaultEditor::new();

    match rl {
        Ok(mut rl) => {
            loop {
                let prompt = "scala> ";
                match rl.readline(prompt) {
                    Ok(line) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        if trimmed == ":quit" || trimmed == ":q" {
                            break;
                        }
                        if trimmed == ":help" {
                            println!("Commands:");
                            println!("  :quit, :q    Exit the REPL");
                            println!("  :help        Show this help");
                            println!("  :reset       Reset the environment");
                            println!("  :type <expr> Show the type of an expression");
                            continue;
                        }
                        if trimmed == ":reset" {
                            interp = Interpreter::new();
                            println!("Environment reset.");
                            continue;
                        }

                        let full_input = if needs_continuation(&line) {
                            let mut buf = line.clone();
                            loop {
                                match rl.readline("     | ") {
                                    Ok(cont) => {
                                        buf.push_str("\n");
                                        buf.push_str(&cont);
                                        if !needs_continuation(&cont) && !buf.trim().ends_with('{') {
                                            break;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                            buf
                        } else {
                            line.clone()
                        };

                        let _ = rl.add_history_entry(&full_input);

                        match interp.run_source(&full_input) {
                            Ok(Value::Unit) => {}
                            Ok(v) => println!("{}", v),
                            Err(e) => eprintln!("{}", e),
                        }
                    }
                    Err(rustyline::error::ReadlineError::Interrupted) => continue,
                    Err(rustyline::error::ReadlineError::Eof) => break,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        break;
                    }
                }
            }
        }
        Err(_) => {
            eprintln!("Warning: line editing not available, using basic input");
            run_basic_repl(&mut interp);
        }
    }
}

fn needs_continuation(input: &str) -> bool {
    let open = input.chars().filter(|&c| c == '{' || c == '(' || c == '[').count();
    let close = input.chars().filter(|&c| c == '}' || c == ')' || c == ']').count();
    if open > close {
        return true;
    }
    let trimmed = input.trim();
    trimmed.ends_with("def")
        || trimmed.ends_with("val")
        || trimmed.ends_with("var")
        || trimmed.ends_with("if")
        || trimmed.ends_with("else")
        || trimmed.ends_with("match")
        || trimmed.ends_with("case")
        || trimmed.ends_with("=>")
        || trimmed.ends_with("=")
        || trimmed.ends_with("while")
        || trimmed.ends_with("for")
        || trimmed.ends_with("yield")
        || trimmed.ends_with("try")
        || trimmed.ends_with("catch")
        || trimmed.ends_with("finally")
        || trimmed.ends_with("class")
        || trimmed.ends_with("trait")
        || trimmed.ends_with("object")
        || trimmed.ends_with("new")
        || trimmed.ends_with("extends")
        || trimmed.ends_with("with")
}

fn run_basic_repl(interp: &mut Interpreter) {
    use std::io::{self, Write, BufRead};
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("scala> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        match stdin.lock().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == ":quit" || trimmed == ":q" {
            break;
        }

        match interp.run_source(trimmed) {
            Ok(Value::Unit) => {}
            Ok(v) => println!("{}", v),
            Err(e) => eprintln!("{}", e),
        }
    }
}
