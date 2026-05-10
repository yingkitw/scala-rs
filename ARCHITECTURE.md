# scala Architecture

## Overview

scala is a Scala language interpreter implemented in Rust. It processes Scala source code through a classic compiler pipeline: **Lexing -> Parsing -> Type Checking -> Interpretation**.

```
┌──────────┐    ┌──────────┐    ┌──────────────┐    ┌─────────────┐
│  Source   │───>│  Lexer   │───>│   Parser     │───>│ Typechecker │
│  (text)   │    │ (tokens) │    │   (AST)      │    │  (typed AST)│
└──────────┘    └──────────┘    └──────────────┘    └──────┬──────┘
                                                           │
                                                           v
                                                   ┌──────────────┐
                                                   │ Interpreter  │
                                                   │  (values)    │
                                                   └──────────────┘
```

## Module Structure

### `token.rs` — Token Types

Defines all token variants produced by the lexer:
- Keywords (val, var, def, class, trait, object, if, match, etc.)
- Operators (+, -, *, /, ==, ::, =>, ->, etc.)
- Literals (Int, Long, Double, Float, String, Char, Boolean, Null)
- Identifiers (alphanumeric, symbolic, backtick-quoted)
- Delimiters and brackets
- Interpolation start/continue/end
- EOF

Each token carries a `Span` (line, column, offset) for error reporting.

### `lexer.rs` — Tokenizer

Converts raw source text into a `Vec<Token>`. Key responsibilities:
- Unicode-aware character classification
- Number parsing with suffix handling (L, f, d)
- String parsing with escape sequence processing
- Multi-line string support
- String interpolation state machine
- Nested block comment tracking
- Operator disambiguation (e.g., `:` vs `::`, `<` vs `<:`)
- Whitespace and comment skipping while tracking positions

Design: State machine iterating over chars with lookahead.

### `ast.rs` — Abstract Syntax Tree

The central data structures representing parsed Scala code. Uses arena-free, owned types.

**Expressions (`Expr` enum):**
- `Literal` — Int, Long, Float, Double, Bool, String, Char, Null, Unit
- `Binary` — left op right (all operators)
- `Unary` — prefix or postfix (negate, not)
- `If` — condition, then, optional else
- `Block` — sequence of statements, last is result
- `Lambda` — parameters, body
- `Apply` — function(args)
- `MethodCall` — receiver.method(args)
- `FieldAccess` — receiver.field
- `Match` — scrutinee, list of cases (pattern -> body)
- `Tuple` — list of expressions
- `Assign` — target = value
- `Return` — optional value
- `Throw` — exception expression
- `Try` — body, catches, finally
- `New` — class instantiation
- `For` — enumerators, yield body or imperative body
- `While` — condition, body
- `StringInterpolation` — parts and expressions
- `This`, `Super`, `Wildcard`
- `Paren` — wrapped expression

**Declarations (`Decl` enum):**
- `Val` — name, type annotation, value
- `Var` — name, type annotation, value
- `Def` — name, type params, params, return type, body
- `Class` — name, type params, primary ctor, parents, body
- `Trait` — name, type params, parents, body
- `Object` — name, parents, body
- `CaseClass` — name, type params, ctor params, parents, body
- `TypeDef` — name, type params, rhs
- `Import` — path with selectors

**Patterns (`Pattern` enum):**
- `Wildcard`, `Variable`, `Literal`, `Constructor`, `Tuple`, `Typed`, `Alternative`, `SequenceWildcard`

**Types (`TypeExpr` enum):**
- Simple, Parameterized, Function, Tuple, Compound

All nodes carry `Span` for source location.

### `parser.rs` — Recursive Descent Parser

Converts `Vec<Token>` into `Vec<Decl>` (top-level) or `Expr`.

Uses a **recursive descent** strategy with:
- Token cursor (current position, lookahead)
- Precedence climbing for binary expressions
- Distinct entry points for different precedence levels
- Error recovery: synchronize on statement boundaries

**Parsing order (high to low precedence):**
1. Assignment
2. Or (`||`)
3. And (`&&`)
4. Comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`)
5. Range (`|`)
6. Concat (`+`, `-` in some contexts)
7. Additive (`+`, `-`)
8. Multiplicative (`*`, `/`, `%`)
9. Shift (`<<`, `>>`, `>>>`)
10. Bitwise (`&`, `^`, `|`)
11. Unary (`!`, `-`, `+`, `~`)
12. Postfix (method call, field access, apply)
13. Primary (literals, grouping, lambda, if, match, etc.)

### `ty.rs` — Type Representation

Runtime type representation used during type checking:
- `TyInt`, `TyLong`, `TyDouble`, `TyFloat`, `TyBool`, `TyChar`, `TyString`, `TyUnit`
- `TyAny`, `TyAnyVal`, `TyAnyRef`, `TyNothing`, `TyNull`
- `TyFunction(params, return)`, `TyTuple(elements)`
- `TyNamed(name)`, `TyParam(name)`, `TyApp(base, args)`
- `TyError` for error recovery

Implements `SubtypeOf` for subtype checking.

### `typechecker.rs` — Type Checker

Walks the AST and assigns types. Key operations:
- **Environment**: Stack of scopes mapping names to types
- **Expression typing**: Bottom-up, each expression produces a `Ty`
- **Declaration typing**: Processes class/trait/object/def/val/var
- **Type inference**: For val/var without type annotations, infers from initializer. For def without return type, infers from body (non-recursive only).
- **Subtype checking**: Verifies assignments and method calls conform
- **Generic instantiation**: Resolves type parameters at call sites
- **Pattern checking**: Verifies pattern types match scrutinee

Produces a typed AST or type errors with spans.

### `value.rs` — Runtime Values

Runtime representation of evaluated expressions:
- `VInt(i64)`, `VLong(i64)`, `VDouble(f64)`, `VFloat(f64)`
- `VBool(bool)`, `VChar(char)`
- `VString(String)`
- `VUnit`
- `VNull`
- `VTuple(Vec<Value>)`
- `VList(Vec<Value>)`
- `VMap(Vec<(Value, Value)>)`
- `VFunction(params, body, captured_env)` — closures
- `VObject(class_name, fields, methods, trait_mixins)`
- `VArray(Vec<Value>)`

Implements `Display` for REPL output.

### `env.rs` — Environment

Stack-based scoping for the interpreter:
```
┌───────────────────┐
│  Global Scope     │  <- builtins, stdlib
├───────────────────┤
│  Module Scope     │  <- top-level declarations
├───────────────────┤
│  Function Scope   │  <- function parameters, locals
├───────────────────┤
│  Block Scope      │  <- val/var in blocks
└───────────────────┘
```

- Each scope is a `HashMap<String, (Value, bool)>` where `bool` = mutable
- `define(name, value, mutable)` — add binding
- `lookup(name)` — walk up scopes
- `assign(name, value)` — find and update (mutable only)
- `push()` / `pop()` — scope management
- Closure capture: clone the environment chain at lambda definition time

### `interpreter.rs` — Tree-Walking Evaluator

Evaluates typed AST to produce `Value`s.

**Evaluation strategy:**
- Strict/eager evaluation of all expressions
- Environment passed through evaluation
- Closures capture environment at definition time
- Classes define constructor functions + method tables
- Traits provide method dictionaries
- Pattern matching is destructuring + guards

**Key methods:**
- `eval_expr(expr, env) -> Value`
- `exec_decl(decl, env) -> ()`
- `eval_pattern(pattern, value, env) -> bool` (binds variables on match)
- `call_function(fn, args, env) -> Value`
- `instantiate_class(name, args, env) -> Value`

### `repl.rs` — Interactive REPL

Uses `rustyline` for line editing:
- Reads input, detects multi-line (unclosed brackets)
- Parses each entry (expression or declaration)
- Type-checks
- Evaluates
- Prints result and type
- Maintains persistent environment across entries

### `stdlib.rs` — Standard Library

Populates the global environment with built-in bindings:

**Predef functions:**
- `println`, `print` — output to stdout
- `assert`, `require` — runtime assertions
- `identity` — returns its argument

**List operations:**
- `List.apply(elements)` — factory
- Methods via method dispatch: `map`, `filter`, `foldLeft`, `foldRight`, `head`, `tail`, etc.

**Option operations:**
- `Some(value)`, `None`
- Methods: `map`, `flatMap`, `get`, `getOrElse`, `isDefined`

**Math operations:**
- `math.Pi`, `math.E`
- `math.abs`, `math.max`, `math.min`, `math.pow`, `math.sqrt`

## Error Handling

All errors carry a `Span` and are reported with source context:

```
error: type mismatch
  --> script.scala:3:8
   |
 3 |  val x: String = 42
   |                   ^^
   |                   found:    Int
   |                   expected: String
```

Error types:
- `LexError` — invalid tokens, unterminated strings
- `ParseError` — unexpected tokens, missing delimiters
- `TypeError` — type mismatches, unbound names, incompatible patterns
- `RuntimeError` — division by zero, null dereference, match errors, index out of bounds

## Design Decisions

1. **Tree-walking interpreter** over bytecode VM — simpler to implement, sufficient for the target use case
2. **Recursive descent parser** over parser generators — more control, easier error messages
3. **Arena-free AST** — owned types with `Clone` where needed, simpler memory management in Rust
4. **Separate type checker** — enables `--check` mode and catches errors before evaluation
5. **Method dispatch at runtime** — flexible for dynamic Scala patterns, trades some performance
6. **Environment cloning for closures** — correct semantics, acceptable for interpreted mode
