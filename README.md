# scala-rs

A Scala language implementation written in Rust. An interpreter for a practical subset of Scala, featuring type inference, pattern matching, classes, traits, and a REPL.

## Features

- **Lexer & Parser** — Full tokenization and recursive-descent parsing of Scala syntax
- **AST** — Typed abstract syntax tree with source location tracking
- **Type System** — Type checking with local type inference, generics, and trait resolution
- **Interpreter** — Tree-walking evaluator with environments and closures
- **REPL** — Interactive read-eval-print loop with multi-line input
- **Standard Library** — Built-in types: `Int`, `Long`, `Double`, `Float`, `Boolean`, `String`, `Unit`, `List`, `Map`, `Option`, `Tuple`

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
# Run a Scala file
cargo run -- script.scala

# Start the REPL
cargo run -- --repl

# Check types only
cargo run -- --check script.scala
```

## Running Tests

```bash
cargo test
```

## Project Structure

```
scala-rs/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library root
│   ├── lexer.rs         # Tokenizer
│   ├── token.rs         # Token types
│   ├── ast.rs           # Abstract Syntax Tree
│   ├── parser.rs        # Recursive-descent parser
│   ├── ty.rs            # Type representation
│   ├── typechecker.rs   # Type checking & inference
│   ├── interpreter.rs   # Tree-walking evaluator
│   ├── repl.rs          # Interactive REPL
│   ├── env.rs           # Environment / scoping
│   ├── value.rs         # Runtime values
│   └── stdlib.rs        # Built-in functions & types
├── tests/               # Integration tests
├── SPEC.md              # Language specification
├── TODO.md              # Implementation roadmap
└── ARCHITECTURE.md      # Architecture design
```

## License

MIT
