pub mod token;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod ty;
pub mod typechecker;
pub mod value;
pub mod env;
pub mod interpreter;
pub mod stdlib;
pub mod repl;

pub fn run_file(source: &str, check_only: bool, dump_tokens: bool, dump_ast: bool) -> Result<(), String> {
    let tokens = lexer::Lexer::tokenize(source)
        .map_err(|e| format!("lexer error: {}", e))?;

    if dump_tokens {
        for token in &tokens {
            println!("{:?}", token);
        }
        return Ok(());
    }

    let stmts = parser::Parser::parse(tokens)
        .map_err(|e| format!("parse error: {}", e))?;

    if dump_ast {
        for stmt in &stmts {
            println!("{}", ast::fmt_stmt(stmt, 0));
        }
        return Ok(());
    }

    if check_only {
        typechecker::typecheck_program(&stmts)
            .map_err(|errs| {
                errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
            })?;
        println!("Type check passed.");
        return Ok(());
    }

    let mut interp = interpreter::Interpreter::new();
    let _ = interp.run_source(source)
        .map_err(|e| format!("{}", e))?;

    Ok(())
}
