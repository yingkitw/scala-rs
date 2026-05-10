//! Tests that distinguish **compilation** (lex + parse + typecheck) from
//! **interpretation** (evaluation via `Interpreter::run_source`).
//!
//! **`compile_success_and_run`** enforces both gates when a program must
//! statically check and execute without error (the user's “compiled and able
//! to run” contract).
//!
//! The interpreter does not run the typechecker, so other tests document when
//! static checking and runtime disagree (e.g. `3 * true`).

use scala::interpreter::Interpreter;
use scala::typechecker;
use scala::value::Value;

fn compile(source: &str) -> Result<(), String> {
    typechecker::typecheck_source(source).map_err(|errs| {
        errs.iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn interpret(source: &str) -> Result<Value, String> {
    Interpreter::new()
        .run_source(source)
        .map_err(|e| e.to_string())
}

/// Typecheck passes **and** the interpreter runs the same source to completion.
fn compile_success_and_run(source: &str) -> Result<Value, String> {
    if let Err(e) = compile(source) {
        return Err(format!("compile failed:\n{e}"));
    }
    interpret(source).map_err(|e| format!("compile ok but run failed:\n{e}"))
}

#[track_caller]
fn assert_compile_success_and_runs(source: &str) -> Value {
    compile_success_and_run(source).unwrap_or_else(|msg| panic!("{msg}\nsource:\n{source}"))
}

fn assert_compile_err_contains(source: &str, needle: &str) {
    let err = compile(source).expect_err("expected compile failure");
    assert!(
        err.contains(needle),
        "compile error should mention {needle:?}, got:\n{err}"
    );
}

#[test]
fn compile_success_and_runnable_literals_and_arithmetic() {
    let cases = [
        ("42", Value::Int(42)),
        ("\"scala\"", Value::String("scala".into())),
        ("true", Value::Bool(true)),
        ("false", Value::Bool(false)),
        ("()", Value::Unit),
        ("null", Value::Null),
        ("1 + 2 * 3", Value::Int(7)),
        ("-(3 + 4)", Value::Int(-7)),
    ];
    for (src, want) in cases {
        let got = assert_compile_success_and_runs(src);
        assert_eq!(got, want, "wrong value for source {src:?}");
    }
}

#[test]
fn compile_success_and_runnable_val_def_control_flow_and_match() {
    let prog = concat!(
        "val n = 6\n",
        "val m = 7\n",
        "def add(a: Int, b: Int): Int = a + b\n",
        "val s = add(n, m)\n",
        "if (s > 10) s match { case 13 => 100 case _ => 0 } else 0\n",
    );
    assert_eq!(assert_compile_success_and_runs(prog), Value::Int(100));

    // Recursive calls are not typed with a forward-binding rule yet (`--check`
    // reports not found inside the body). Use an equivalent closed form instead.
    let closed = concat!(
        "def triangular(n: Int): Int = n * (n + 1) / 2\n",
        "triangular(10)\n",
    );
    assert_eq!(assert_compile_success_and_runs(closed), Value::Int(55));
}

#[test]
fn compile_success_and_runnable_for_comp_while_lists() {
    let for_src = concat!(
        "val xs = for (x <- List(2, 3, 4)) yield x * x\n",
        "xs.length\n",
    );
    assert_eq!(assert_compile_success_and_runs(for_src), Value::Int(3));

    let src = concat!(
        "var acc = 0\n",
        "var i = 1\n",
        "while (i <= 5) {\n",
        "  acc = acc + i\n",
        "  i = i + 1\n",
        "}\n",
        "acc\n",
    );
    assert_eq!(assert_compile_success_and_runs(src), Value::Int(15));

    // Curried `.foldLeft(0)((a,b) => …)` parses as fewer args here; reduce matches runtime.
    assert_eq!(
        assert_compile_success_and_runs(
            "List(10, 20, 30).reduce((a: Int, b: Int) => a + b)\n",
        ),
        Value::Int(60),
    );
}

#[test]
fn compile_success_and_runnable_trait_class_field_access() {
    let src = r#"
trait Shape { def side: Double }
class Square(val side: Double) extends Shape {}
val sq = new Square(9.0)
sq.side * sq.side / 27.0
"#;
    match assert_compile_success_and_runs(src) {
        Value::Double(d) => assert!((d - 3.0).abs() < 1e-9),
        Value::Float(f) => assert!((f - 3.0).abs() < 1e-9),
        other => panic!("expected numeric side result, got {other:?}"),
    }
}

#[test]
fn compile_rejects_illegal_unary_operations() {
    assert_compile_err_contains("-true", "cannot negate Boolean");
    assert_compile_err_contains("!42", "! requires Boolean");
    assert_compile_err_contains("~true", "~ requires numeric");
}

#[test]
fn compile_rejects_non_boolean_condition() {
    assert_compile_err_contains("if (1) 2 else 3", "if condition must be Boolean");
}

#[test]
fn compile_rejects_unknown_identifier() {
    assert_compile_err_contains("notDefinedAnywhere42", "not found: value");
}

#[test]
fn compile_rejects_def_body_return_type() {
    assert_compile_err_contains(
        "def foo(x: Int): String = x + 1",
        "return type mismatch",
    );
}

#[test]
fn compile_rejects_parse_errors() {
    assert_compile_err_contains("val x =", "unexpected token");
}

#[test]
fn compile_accepts_but_interpret_reports_division_by_zero() {
    let src = "1 / 0";
    compile(src).expect("typechecker allows integral division operands");
    let err = interpret(src).expect_err("expected runtime division error");
    assert!(err.contains("division by zero"), "unexpected error:\n{err}");
}

#[test]
fn compile_accepts_but_interpret_reports_numeric_op_type_mismatch() {
    let src = "3 * true";
    compile(src).expect("typechecker is permissive here; runtime guards types");
    let err = interpret(src).expect_err("expected runtime error");
    assert!(err.contains("cannot multiply"), "unexpected error:\n{err}");
}

#[test]
fn interpreter_does_not_require_separate_compile_step() {
    // Same source passes both when it is fully well-typed in practice.
    let src = "def double(x: Int): Int = x * 2; double(21)";
    assert_eq!(assert_compile_success_and_runs(src), Value::Int(42));

    let bad = compile("badIdent999");
    assert!(bad.is_err());
    let interp_err = interpret("badIdent999").expect_err("runtime should still fail");
    assert!(
        interp_err.contains("not found"),
        "expected runtime not found:\n{interp_err}",
    );
}

#[test]
fn compile_success_and_run_reports_which_stage_failed() {
    let type_err = compile_success_and_run("if (1) 2 else 3").expect_err("should not pass");
    assert!(
        type_err.starts_with("compile failed:"),
        "expected compile stage in message, got:\n{type_err}",
    );

    let run_err = compile_success_and_run("1 / 0").expect_err("runtime error");
    assert!(
        run_err.starts_with("compile ok but run failed:"),
        "expected run stage in message, got:\n{run_err}",
    );
    assert!(run_err.contains("division by zero"), "got:\n{run_err}");
}
