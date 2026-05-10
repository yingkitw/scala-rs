# scala Architecture

## Overview

scala is a Scala language interpreter implemented in Rust. Sources always go through **lexing**, **parsing**, and **interpretation** when you run programs. **Type-checking is optional** depending on entry point: `--check` and **`typecheck_then_run`** run the checker first; ordinary file execution does not (unless you choose **`--verify-types`**).

```
┌──────────┐    ┌──────────┐    ┌──────────────┐    ┌─────────────┐
│  Source   │───>│  Lexer   │───>│   Parser      │──>│ Interpreter │
│  (text)   │    │ (tokens) │    │   (AST)       │    │   (values)  │
└──────────┘    └──────────┘    └──────────────┘    └─────────────┘
                                       │optional
                                       v
                               ┌───────────────┐
                               │ Typechecker   │
                               └───────────────┘
```

### Execution modes (CLI and library)

| Entry | Type-check before run? |
|-------|-------------------------|
| `scala <file.scala>` — `scala::run_file(_, false, …)` | No |
| `scala --check <file>` — `scala::run_file(_, true, …)` | Yes (then exit; no interpreter) |
| `scala --verify-types <file>` — `scala::typecheck_then_run` | Yes, then interpreter |
| `scala::interpret_source` | No |

`--tokens` / `--ast` dumps short-circuit after lex or parse respectively (see `lib.rs`).

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

The central data structures use owned types (`Clone` where needed).

Programs are **`Vec<Stmt>`** at the top level.

**Statements (`Stmt`):**
- **`Expr`** — bare expression statement
- **`ValDecl`** / **`VarDecl`** — bindings with optional type and pattern
- **`DefDecl`** — methods and top-level functions (name, params, optional return **`TypeExpr`**, body **`Expr`**)
- **`ClassDecl`** / **`TraitDecl`** / **`ObjectDecl`** — type and module scaffolding with nested **`Vec<Stmt>`** bodies
- **`TypeDecl`** — type aliases
- **`ImportDecl`** — import paths / selectors

**Expressions (`Expr`)** — non-exhaustive list:
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

**Patterns (`Pattern` enum):**
- `Wildcard`, `Variable`, `Literal`, `Constructor`, `Tuple`, `Typed`, `Alternative`, `SequenceWildcard`

**Types (`TypeExpr` enum):**
- Simple, Parameterized, Function, Tuple, Compound

All nodes carry `Span` for source location.

### `parser.rs` — Recursive Descent Parser

Converts `Vec<Token>` into `Vec<Stmt>` (program) or a standalone **`Expr`** via **`Parser::parse_expr`**.

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

### `lib.rs` — Public API

Thin wrappers used by the CLI and integration tests:

- **`run_file`** — tokenize, optionally dump tokens/AST, optionally `typecheck_program`, or run `Interpreter::run_source` on full text.
- **`typecheck_then_run`** — `typecheck_source` then a fresh interpreter (`Result<Value, String>`).
- **`interpret_source`** — parse + eval only (`Result<Value, RuntimeError>`).

The interpreter does not retain a typed IR; “type checking” is a separate AST pass that mirrors what `typecheck_source` does on the same source text.

### `ty.rs` — Type representation

Static types used only in **`typechecker`** (not stored on runtime values):

- Primitives: `Int`, `Long`, `Double`, `Float`, `Bool`, `Char`, `String`, `Unit`
- Top / bottom: `Any`, `AnyVal`, `AnyRef`, `Nothing`, `Null`
- `Function { params, result }`, `Tuple { elements }`
- `Named { name, args }`, `Param`, `App`
- `Error(String)` for propagated errors

`TypeEnv` is a stack of `HashMap<String, Ty>` scopes (see `lookup` / `define` / `push` / `pop`).

### `typechecker.rs` — Type checker

Walks the AST with a `TypeEnv` and returns `Ty` or **`TypeError`** with spans:

- **Declarations:** `val`, `var`, **`def`**, `class`, `trait`, **`object`**. Top-level **`object` names** are registered in the enclosing scope after the object body is checked so **`O.method`** resolves in later statements.
- **Self-recursion on `def`:** the function name is **forward-declared** with a stub (`Function` whose result is the declared return type, or `Any` if inferred) before the parameter frame is pushed, so bodies like `fact(n - 1)` type-check when an explicit return type is present.
- Expression typing is partial: some operations fall back to **`Any`** where the implementation is intentionally loose.

`typecheck_source` lexes and parses, then runs `typecheck_program`.

### `value.rs` — Runtime values

Interpreted data (`Value`): numeric types, `String`, `Bool`, `Char`, `Unit`, `Null`, `Nothing`; `Tuple`, `List`, `Map`, `Array`; closures (`Function` / `BuiltinFunction`); heap objects (`Object`). Implements `Display` for REPL output. Static types (`Ty`) are **not** stored on values.

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

### `interpreter.rs` — Tree-walking evaluator

Evaluates the AST to `Value`s (there is no separate typed IR consumed at runtime). Strategy: strict evaluation, `Environment` stack, closures capture bindings at definition time, classes/objects use ctor + method tables, pattern matching with optional guards.

Entry point used by CLI and tests: **`Interpreter::run_source`** (internally lex, parse, then `exec_stmt` for each top-level statement). Other logic is centered on **`eval_expr`**, **`exec_stmt`**, dispatch, and pattern binding.

### `repl.rs` — Interactive REPL

Uses **`rustyline`** when available, otherwise stdin. Multi-line heuristic from bracket balance and keyword suffixes.

Each input is **`Interpreter::run_source`** over that snippet only; nested environments persist across lines.

Commands: **`:help`**, **`:quit`** / **`:q`**, **`:reset`**, **`:type &lt;expr&gt;`**. `:type` runs **`typechecker::typecheck_expr_standalone`** with a **new** prelude-only **`TypeEnv`** — it does **not** see `val` / `def` from earlier REPL submissions (see **`TODO.md`** for full-environment `:type`).

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
4. **Separate type checker** — enables `--check`, library `typecheck_then_run`, and `--verify-types`; default file execution stays parse-then-interpret for compatibility
5. **Method dispatch at runtime** — flexible for dynamic Scala patterns, trades some performance
6. **Environment cloning for closures** — correct semantics, acceptable for interpreted mode
