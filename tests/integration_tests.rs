use scala_rs::interpreter::Interpreter;
use scala_rs::value::Value;
use scala_rs::lexer::Lexer;
use scala_rs::parser::Parser;
use scala_rs::typechecker;

fn eval(source: &str) -> Value {
    let mut interp = Interpreter::new();
    interp.run_source(source).unwrap()
}

fn eval_err(source: &str) -> String {
    let mut interp = Interpreter::new();
    interp.run_source(source).unwrap_err().message
}

fn lexes(source: &str) -> bool {
    Lexer::tokenize(source).is_ok()
}

fn parses(source: &str) -> bool {
    let tokens = Lexer::tokenize(source).unwrap();
    Parser::parse(tokens).is_ok()
}

fn typechecks(source: &str) -> bool {
    let tokens = Lexer::tokenize(source).unwrap();
    let stmts = Parser::parse(tokens).unwrap();
    typechecker::typecheck_program(&stmts).is_ok()
}

// ==================== LEXER TESTS ====================

#[test]
fn integration_lexer_all_number_formats() {
    assert!(lexes("42"));
    assert!(lexes("0xFF"));
    assert!(lexes("0b1010"));
    assert!(lexes("3.14"));
    assert!(lexes("1e10"));
    assert!(lexes("42L"));
    assert!(lexes("3.14f"));
    assert!(lexes("0"));
}

#[test]
fn integration_lexer_all_string_formats() {
    assert!(lexes("\"hello\""));
    assert!(lexes("\"hello\\nworld\""));
    assert!(lexes("\"\"\"multi\nline\"\"\""));
    assert!(lexes("'a'"));
    assert!(lexes("'\\n'"));
}

#[test]
fn integration_lexer_operators() {
    assert!(lexes("1 + 2 - 3 * 4 / 5 % 6"));
    assert!(lexes("1 == 2 != 3 < 4 > 5 <= 6 >= 7"));
    assert!(lexes("true && false || !true"));
    assert!(lexes("x => x + 1"));
    assert!(lexes("x <- List(1, 2)"));
    assert!(lexes("a << 1 >> 2"));
}

#[test]
fn integration_lexer_comments() {
    assert!(lexes("// comment\n42"));
    assert!(lexes("/* block */ 42"));
    assert!(lexes("/* outer /* inner */ */ 42"));
}

// ==================== PARSER TESTS ====================

#[test]
fn integration_parser_val_var() {
    assert!(parses("val x = 42"));
    assert!(parses("val x: Int = 42"));
    assert!(parses("var x = 42"));
    assert!(parses("var x: Int = 42"));
}

#[test]
fn integration_parser_def() {
    assert!(parses("def f: Int = 42"));
    assert!(parses("def f(x: Int): Int = x + 1"));
    assert!(parses("def f(x: Int, y: Int): Int = x + y"));
}

#[test]
fn integration_parser_class() {
    assert!(parses("class Foo"));
    assert!(parses("class Foo(x: Int, y: Int)"));
    assert!(parses("class Foo(val x: Int, val y: Int)"));
    assert!(parses("case class Foo(x: Int, y: Int)"));
}

#[test]
fn integration_parser_trait_object() {
    assert!(parses("trait Foo { def bar: Int }"));
    assert!(parses("object Foo { val x = 1 }"));
}

#[test]
fn integration_parser_expressions() {
    assert!(parses("1 + 2 * 3"));
    assert!(parses("(1 + 2) * 3"));
    assert!(parses("if (true) 1 else 2"));
    assert!(parses("{ val x = 1; x + 1 }"));
    assert!(parses("(x) => x + 1"));
    assert!(parses("(1, 2, 3)"));
    assert!(parses("x match { case 1 => 10 case _ => 0 }"));
    assert!(parses("new Foo(1, 2)"));
    assert!(parses("for (x <- List(1)) yield x"));
    assert!(parses("while (true) 1"));
}

// ==================== TYPE CHECKER TESTS ====================

#[test]
fn integration_typecheck_literals() {
    assert!(typechecks("42"));
    assert!(typechecks("\"hello\""));
    assert!(typechecks("true"));
    assert!(typechecks("null"));
    assert!(typechecks("()"));
}

#[test]
fn integration_typecheck_arithmetic() {
    assert!(typechecks("1 + 2"));
    assert!(typechecks("1 + 2 * 3"));
    assert!(typechecks("\"hello\" + \" \" + \"world\""));
}

#[test]
fn integration_typecheck_bindings() {
    assert!(typechecks("val x = 42"));
    assert!(typechecks("val x: Int = 42"));
    assert!(typechecks("var x = 1; x = 2"));
    assert!(typechecks("def f(x: Int): Int = x + 1"));
}

#[test]
fn integration_typecheck_control_flow() {
    assert!(typechecks("if (true) 1 else 2"));
    assert!(typechecks("{ val x = 1; x + 1 }"));
    assert!(typechecks("1 match { case 1 => 10 case _ => 0 }"));
}

#[test]
fn integration_typecheck_functions() {
    assert!(typechecks("(x: Int) => x + 1"));
    assert!(typechecks("def f(x: Int): Int = x + 1; f(42)"));
}

// ==================== INTERPRETER TESTS ====================

#[test]
fn integration_arithmetic_all() {
    assert_eq!(eval("1 + 2"), Value::Int(3));
    assert_eq!(eval("10 - 3"), Value::Int(7));
    assert_eq!(eval("4 * 5"), Value::Int(20));
    assert_eq!(eval("10 / 3"), Value::Int(3));
    assert_eq!(eval("10 % 3"), Value::Int(1));
    assert_eq!(eval("-42"), Value::Int(-42));
    assert_eq!(eval("-(3 + 4)"), Value::Int(-7));
}

#[test]
fn integration_string_operations() {
    assert_eq!(eval("\"hello\".length"), Value::Int(5));
    assert_eq!(eval("\"hello\".toUpperCase"), Value::String("HELLO".into()));
    assert_eq!(eval("\"hello\".toLowerCase"), Value::String("hello".into()));
    assert_eq!(eval("\"  hi  \".trim"), Value::String("hi".into()));
    assert_eq!(eval("\"hello world\".contains(\"world\")"), Value::Bool(true));
    assert_eq!(eval("\"hello world\".split(\" \").length"), Value::Int(2));
    assert_eq!(eval("\"hello\".replace(\"l\", \"r\")"), Value::String("herro".into()));
    assert_eq!(eval("\"hello\" + \" \" + \"world\""), Value::String("hello world".into()));
    assert_eq!(eval("\"hello\".indexOf(\"l\")"), Value::Int(2));
    assert_eq!(eval("\"hello\".reverse"), Value::String("olleh".into()));
}

#[test]
fn integration_list_operations() {
    assert_eq!(eval("List(1, 2, 3).length"), Value::Int(3));
    assert_eq!(eval("List(1, 2, 3).head"), Value::Int(1));
    assert_eq!(eval("List(1, 2, 3).last"), Value::Int(3));
    assert_eq!(eval("List(1, 2, 3).isEmpty"), Value::Bool(false));
    assert_eq!(eval("List().isEmpty"), Value::Bool(true));
    assert_eq!(eval("List(1, 2, 3).contains(2)"), Value::Bool(true));
    assert_eq!(eval("List(1, 2, 3).contains(5)"), Value::Bool(false));
    assert_eq!(eval("List(3, 1, 2).sorted.length"), Value::Int(3));
    assert_eq!(eval("List(1, 2, 3).reverse.length"), Value::Int(3));
}

#[test]
fn integration_list_higher_order() {
    match eval("List(1, 2, 3).map((x) => x * 2)") {
        Value::List(v) => assert_eq!(v, vec![Value::Int(2), Value::Int(4), Value::Int(6)]),
        _ => panic!("expected list"),
    }
    match eval("List(1, 2, 3, 4, 5).filter((x) => x > 3)") {
        Value::List(v) => assert_eq!(v, vec![Value::Int(4), Value::Int(5)]),
        _ => panic!("expected list"),
    }
    assert_eq!(eval("List(1, 2, 3).reduce((a, b) => a + b)"), Value::Int(6));
    assert_eq!(eval("List(1, 2, 3).exists((x) => x > 2)"), Value::Bool(true));
    assert_eq!(eval("List(1, 2, 3).forall((x) => x > 0)"), Value::Bool(true));
    assert_eq!(eval("List(1, 2, 3).forall((x) => x > 2)"), Value::Bool(false));
    assert_eq!(eval("List(1, 2, 3, 4, 5).count((x) => x % 2 == 0)"), Value::Int(2));
    assert_eq!(eval("List(1, 2, 3).mkString(\", \")"), Value::String("1, 2, 3".into()));
    assert_eq!(eval("List(1, 2, 3, 4, 5).sum"), Value::Int(15));
    assert_eq!(eval("List(1, 2, 3, 4, 5).product"), Value::Int(120));
}

#[test]
fn integration_list_take_drop() {
    match eval("List(1, 2, 3, 4, 5).take(3)") {
        Value::List(v) => assert_eq!(v, vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
        _ => panic!("expected list"),
    }
    match eval("List(1, 2, 3, 4, 5).drop(2)") {
        Value::List(v) => assert_eq!(v, vec![Value::Int(3), Value::Int(4), Value::Int(5)]),
        _ => panic!("expected list"),
    }
}

#[test]
fn integration_control_flow() {
    assert_eq!(eval("if (true) 1 else 2"), Value::Int(1));
    assert_eq!(eval("if (false) 1 else 2"), Value::Int(2));
    assert_eq!(eval("if (true) 42"), Value::Int(42));
    assert_eq!(eval("{ val x = 10; val y = 20; x + y }"), Value::Int(30));
}

#[test]
fn integration_match_expression() {
    assert_eq!(eval("3 match { case 1 => 10 case 2 => 20 case _ => 30 }"), Value::Int(30));
    assert_eq!(eval("1 match { case 1 => 10 case 2 => 20 case _ => 30 }"), Value::Int(10));
    assert_eq!(eval("0 match { case 0 => 100 case _ => 0 }"), Value::Int(100));
}

#[test]
fn integration_for_comprehensions() {
    match eval("for (x <- List(1, 2, 3)) yield x * 2") {
        Value::List(v) => assert_eq!(v, vec![Value::Int(2), Value::Int(4), Value::Int(6)]),
        _ => panic!("expected list"),
    }
    match eval("for (x <- List(1, 2, 3, 4, 5); if x % 2 == 0) yield x") {
        Value::List(v) => assert_eq!(v, vec![Value::Int(2), Value::Int(4)]),
        _ => panic!("expected list"),
    }
    match eval("for (x <- List(1, 2); y <- List(10, 20)) yield x + y") {
        Value::List(v) => assert_eq!(v, vec![Value::Int(11), Value::Int(21), Value::Int(12), Value::Int(22)]),
        _ => panic!("expected list"),
    }
}

#[test]
fn integration_recursion() {
    assert_eq!(eval("def fact(n: Int): Int = if (n <= 1) 1 else n * fact(n - 1); fact(10)"), Value::Int(3628800));
    assert_eq!(eval("def fib(n: Int): Int = if (n <= 1) n else fib(n - 1) + fib(n - 2); fib(10)"), Value::Int(55));
    assert_eq!(eval("def gcd(a: Int, b: Int): Int = if (b == 0) a else gcd(b, a % b); gcd(48, 18)"), Value::Int(6));
}

#[test]
fn integration_class_and_object() {
    let result = eval("class Point(val x: Int, val y: Int); val p = new Point(3, 4); p.x + p.y");
    assert_eq!(result, Value::Int(7));

    let result = eval("object Utils { def double(x: Int): Int = x * 2 }; Utils.double(21)");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn integration_tuples() {
    match eval("(1, \"hello\", true)") {
        Value::Tuple(v) => {
            assert_eq!(v.len(), 3);
            assert_eq!(v[0], Value::Int(1));
            assert_eq!(v[1], Value::String("hello".into()));
            assert_eq!(v[2], Value::Bool(true));
        }
        _ => panic!("expected tuple"),
    }
}

#[test]
fn integration_multi_statement_programs() {
    let src = r#"
        val a = 10
        val b = 20
        val c = a + b
        c
    "#;
    assert_eq!(eval(src), Value::Int(30));

    let src = r#"
        def isEven(n: Int): Boolean = n % 2 == 0
        val nums = List(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
        val evens = nums.filter((x) => isEven(x))
        evens.length
    "#;
    assert_eq!(eval(src), Value::Int(5));
}

#[test]
fn integration_while_loop() {
    let src = r#"
        var sum = 0
        var i = 1
        while (i <= 100) {
            sum = sum + i
            i = i + 1
        }
        sum
    "#;
    assert_eq!(eval(src), Value::Int(5050));
}

#[test]
fn integration_division_by_zero() {
    let err = eval_err("1 / 0");
    assert!(err.contains("division by zero"));
}

#[test]
fn integration_unbound_variable() {
    let err = eval_err("x");
    assert!(err.contains("not found"));
}

#[test]
fn integration_type_annotations() {
    assert_eq!(eval("val x: Int = 42; x"), Value::Int(42));
    assert_eq!(eval("val s: String = \"hi\"; s"), Value::String("hi".into()));
    assert_eq!(eval("val b: Boolean = true; b"), Value::Bool(true));
}

#[test]
fn integration_boolean_full() {
    assert_eq!(eval("true && true"), Value::Bool(true));
    assert_eq!(eval("true && false"), Value::Bool(false));
    assert_eq!(eval("false && true"), Value::Bool(false));
    assert_eq!(eval("true || false"), Value::Bool(true));
    assert_eq!(eval("false || false"), Value::Bool(false));
    assert_eq!(eval("!true"), Value::Bool(false));
    assert_eq!(eval("!false"), Value::Bool(true));
}

#[test]
fn integration_comparison_full() {
    assert_eq!(eval("1 == 1"), Value::Bool(true));
    assert_eq!(eval("1 == 2"), Value::Bool(false));
    assert_eq!(eval("1 != 2"), Value::Bool(true));
    assert_eq!(eval("1 != 1"), Value::Bool(false));
    assert_eq!(eval("1 < 2"), Value::Bool(true));
    assert_eq!(eval("2 < 1"), Value::Bool(false));
    assert_eq!(eval("2 > 1"), Value::Bool(true));
    assert_eq!(eval("1 > 2"), Value::Bool(false));
    assert_eq!(eval("1 <= 1"), Value::Bool(true));
    assert_eq!(eval("1 >= 1"), Value::Bool(true));
    assert_eq!(eval("1 <= 2"), Value::Bool(true));
    assert_eq!(eval("2 >= 1"), Value::Bool(true));
}

#[test]
fn integration_bitwise_operations() {
    assert_eq!(eval("5 & 3"), Value::Int(1));
    assert_eq!(eval("5 | 3"), Value::Int(7));
    assert_eq!(eval("5 ^ 3"), Value::Int(6));
    assert_eq!(eval("1 << 4"), Value::Int(16));
    assert_eq!(eval("16 >> 2"), Value::Int(4));
}

#[test]
fn integration_precedence() {
    assert_eq!(eval("2 + 3 * 4"), Value::Int(14));
    assert_eq!(eval("(2 + 3) * 4"), Value::Int(20));
    assert_eq!(eval("2 * 3 + 4 * 5"), Value::Int(26));
    assert_eq!(eval("10 - 3 - 2"), Value::Int(5));
}

#[test]
fn integration_closure_capture() {
    let src = r#"
        def makeAdder(n: Int): Int => Int = (x) => x + n
        val add5 = makeAdder(5)
        val add10 = makeAdder(10)
        add5(100)
    "#;
    assert_eq!(eval(src), Value::Int(105));
}

#[test]
fn integration_full_program() {
    let src = r#"
        def sumOfSquares(n: Int): Int = {
            var total = 0
            var i = 1
            while (i <= n) {
                total = total + i * i
                i = i + 1
            }
            total
        }
        sumOfSquares(10)
    "#;
    assert_eq!(eval(src), Value::Int(385));
}
