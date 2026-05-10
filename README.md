# scala

A Scala language implementation written in Rust. An interpreter for a practical subset of Scala, featuring type inference, pattern matching, classes, traits, and a REPL.

## Features

- **Lexer & Parser** — Full tokenization and recursive-descent parsing of Scala syntax
- **AST** — Typed abstract syntax tree with source location tracking
- **Type System** — Type checking with local type inference, generics, and trait resolution
- **Interpreter** — Tree-walking evaluator with environments and closures
- **REPL** — Interactive read-eval-print loop with multi-line input and built-in commands (`:help`, `:reset`, prelude-only `:type`)
- **Standard Library** — Built-in types: `Int`, `Long`, `Double`, `Float`, `Boolean`, `String`, `Unit`, `List`, `Map`, `Option`, `Tuple`

## Crate and binary

The Rust package name is **`scala`**. After `cargo build`, the interpreter binary is **`target/debug/scala`** (or **`target/release/scala`** with `--release`).

Embedding or testing from Rust uses the **`scala`** crate: see [Library](#library-api) below.

## Supported Scala Subset

```scala
// Variables
val x = 42
var y = "hello"
y = "world"

// Functions
def add(a: Int, b: Int): Int = a + b
def greet(name: String): Unit = println(s"Hello, $name!")

// Classes and Traits
trait Shape {
  def area: Double
}

class Circle(val radius: Double) extends Shape {
  def area: Double = math.Pi * radius * radius
}

// Case classes and pattern matching
case class Point(x: Int, y: Int)

def describe(p: Point): String = p match {
  case Point(0, 0) => "origin"
  case Point(x, 0) => s"on x-axis at $x"
  case Point(_, y) => s"y=$y"
}

// Collections
val nums = List(1, 2, 3, 4, 5)
val doubled = nums.map(_ * 2)
val sum = nums.foldLeft(0)(_ + _)

// For-comprehensions
val evens = for {
  n <- nums
  if n % 2 == 0
} yield n

// String interpolation
val name = "Scala"
println(s"Hello from $name!")
```

## Building

```bash
cargo build
```

## Running

```bash
# Run a Scala file (lex + parse + interpret; no separate type-check pass by default)
cargo run -- script.scala

# Same as release binary:
#   ./target/release/scala script.scala

# Start the REPL
cargo run -- --repl

# Type-check only (exit 0 if all statements type-check)
cargo run -- --check script.scala

# Type-check, then run (fails before interpretation if compile errors)
cargo run -- --verify-types script.scala

# Debug: dump tokens or pretty-print AST
cargo run -- --tokens script.scala
cargo run -- --ast script.scala

# Version string (matches Cargo.toml)
cargo run -- --version
```

By default, **file execution does not run the typechecker**. Use **`--check`** for static diagnostics only or **`--verify-types`** when you want “compile then run” semantics. Details: [SPEC.md](SPEC.md) (Host tooling).

## Library API

Public helpers on the library root (`src/lib.rs`):

| Function | Behavior |
|---------|----------|
| `scala::run_file(source, …)` | Used by the CLI for `--tokens`, `--ast`, `--check`, and plain execution. |
| `scala::typecheck_then_run(source)` | Lexes, parses, runs `typechecker::typecheck_source`, then interprets with a fresh interpreter. Returns `Result<value::Value, String>`. |
| `scala::interpret_source(source)` | Interprets only (same as `Interpreter::run_source` on a fresh interpreter). |

Integration tests exercise both paths under `tests/`.

## Running Tests

```bash
cargo test
```

## Benchmarks (optional)

Criterion bench for `fib` interpret throughput (requires **`cargo bench`** dev profile):

```bash
cargo bench --bench fib_interp
```

## Project Structure

```
scala/
├── docs/
│   └── index.html       # Links to root Markdown docs
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library root and public wrappers
│   ├── lexer.rs         # Tokenizer
│   ├── token.rs         # Token types
│   ├── ast.rs           # Abstract syntax tree
│   ├── parser.rs        # Recursive-descent parser
│   ├── ty.rs            # Compile-time types (`Ty`)
│   ├── typechecker.rs   # Type checking & inference
│   ├── interpreter.rs   # Tree-walking evaluator
│   ├── repl.rs          # Interactive REPL
│   ├── env.rs           # Interpreter environments
│   ├── value.rs         # Runtime values
│   └── stdlib.rs        # Built-ins and prelude wiring
├── tests/               # Integration and pipeline tests
├── SPEC.md              # Language specification
├── TODO.md              # Implementation roadmap
└── ARCHITECTURE.md      # Component-level design notes
```

## License

Apache-2.0
