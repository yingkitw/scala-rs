# scala Language Specification

This document defines the language subset of Scala that scala implements.

## 1. Lexical Structure

### 1.1 Keywords

```
abstract  case      catch     class     def       do        else
extends   false     final     finally   for       forSome   if
implicit  import    lazy      match     new       null      object
override  package   private   protected return    sealed    super
this      throw     trait     true      try       type      val
var       while     with      yield
```

### 1.2 Operators and Delimiters

```
+  -  *  /  %        arithmetic
&  |  ^  ~           bitwise
=  !  <  >           comparison
<< >> >>>            shift
&& ||                logical short-circuit
== != <= >=          equality/comparison
=> ->                arrow
<-                   for-comprehension
:  ;  .  ,  _        punctuation
(  )  [  ]  {  }     brackets
```

### 1.3 Literals

| Type        | Examples                                    |
|-------------|---------------------------------------------|
| Integer     | `42`, `0xFF`, `0o77`, `0b1010`             |
| Long        | `42L`, `0xFFL`                              |
| Double      | `3.14`, `1.0e10`, `.5`                     |
| Float       | `3.14f`, `1.0f`                            |
| Boolean     | `true`, `false`                             |
| Character   | `'a'`, `'\n'`, `'\u0041'`                  |
| String      | `"hello"`, `"""multi\nline"""`             |
| Null        | `null`                                      |
| Unit        | `()`                                        |
| Symbol      | `'symbol`                                   |

### 1.4 Identifiers

- Alphanumeric: letter followed by letters, digits, or underscores
- Operator: one or more operator characters
- Backtick-quoted: `` `any string` ``

### 1.5 Comments

```scala
// single line
/* block comment */
/** doc comment */
```

### 1.6 String Interpolation

```scala
s"Hello, $name!"
s"Result: ${expr}"
raw"no\nescaping"
f"Pi is $pi%.2f"
```

## 2. Types

### 2.1 Primitive Types

| Type     | Description                |
|----------|----------------------------|
| `Int`    | 32-bit signed integer      |
| `Long`   | 64-bit signed integer      |
| `Double` | 64-bit floating point      |
| `Float`  | 32-bit floating point      |
| `Boolean`| `true` or `false`          |
| `Char`   | 16-bit Unicode character   |
| `String` | UTF-16 string              |
| `Unit`   | No meaningful value        |
| `Any`    | Top of type hierarchy      |
| `AnyVal` | Parent of value types      |
| `AnyRef` | Parent of reference types  |
| `Nothing`| Bottom type                |
| `Null`   | Bottom of reference types  |

### 2.2 Function Types

```scala
(A, B) => C           // Function2[A, B, C]
A => B                // Function1[A, B]
() => A               // Function0[A]
```

### 2.3 Tuple Types

```scala
(A, B)                // Tuple2[A, B]
(A, B, C)             // Tuple3[A, B, C]
```

### 2.4 Parameterized Types

```scala
List[Int]
Map[String, Int]
Option[Double]
Either[String, Int]
```

### 2.5 Type Parameters (Generics)

```scala
class Pair[A, B](val first: A, val second: B)
def identity[T](x: T): T = x
```

Variance annotations:
- `+T` — covariant
- `-T` — contravariant
- `T` — invariant (default)

Type bounds:
- `T <: Upper` — upper bound
- `T >: Lower` — lower bound

## 3. Declarations

### 3.1 Values

```scala
val x = 42
val x: Int = 42
```

Immutable bindings. Single assignment.

### 3.2 Variables

```scala
var x = 42
var x: Int = 42
x = 43
```

Mutable bindings.

### 3.3 Functions

```scala
def f(x: Int): Int = x + 1
def f(x: Int): Int = {          // block body
  val y = x + 1
  y
}
def f(x: Int)(y: Int): Int = x + y   // curried
def f(x: Int*): Int = x.sum           // varargs
```

### 3.4 Type Aliases

```scala
type StringMap[V] = Map[String, V]
```

### 3.5 Imports

```scala
import scala.collection.mutable
import scala.collection.mutable.{HashMap, HashSet}
import scala.collection.mutable._
```

## 4. Expressions

### 4.1 Literals

All literal types listed in section 1.3.

### 4.2 Arithmetic

```scala
a + b   a - b   a * b   a / b   a % b
a | b   a & b   a ^ b
a << n  a >> n  a >>> n
```

### 4.3 Comparison

```scala
a == b  a != b  a < b  a > b  a <= b  a >= b
a eq b  a ne b                            // reference equality
```

### 4.4 Boolean

```scala
a && b  a || b  !a
```

### 4.5 String Concatenation

```scala
"hello" + " " + "world"
```

### 4.6 If/Else

```scala
if (cond) expr
if (cond) expr1 else expr2
```

Both branches must be expressions when used as an expression. The type is the least upper bound of both branches.

### 4.7 Match Expression

```scala
expr match {
  case pattern1 => body1
  case pattern2 if guard2 => body2
  case _ => defaultBody
}
```

Patterns:
- Wildcard: `_`
- Variable: `x`
- Literal: `42`, `"hello"`, `true`
- Constructor: `Point(0, y)`
- Tuple: `(a, b, c)`
- Typed: `x: Int`
- Sequence wildcard: `xs*` (in constructor patterns)
- Alternative: `a | b`
- Stable identifier: `` `x` ``

### 4.8 Block Expression

```scala
{
  val x = 1
  val y = 2
  x + y     // last expression is the result
}
```

### 4.9 Function Literals (Lambdas)

```scala
(x: Int) => x + 1
(x: Int, y: Int) => x + y
_ + 1                    // placeholder syntax
(x: Int) => {
  val y = x + 1
  y
}
```

### 4.10 Method Invocation

```scala
obj.method(arg)
obj method arg            // infix style
obj.method                // no-arg
obj.method()              // empty parens
```

### 4.11 Field Access

```scala
obj.field
```

### 4.12 Tuples

```scala
(1, "hello", true)
```

### 4.13 New Expression

```scala
new ClassName(args)
new ClassName[A, B](args)
new TraitName { body }     // anonymous class
```

### 4.14 Type Cast

```scala
expr.asInstanceOf[Type]
expr.isInstanceOf[Type]
```

### 4.15 Throw / Try / Catch

```scala
throw new Exception("msg")

try {
  expr
} catch {
  case e: IOException => handle(e)
} finally {
  cleanup()
}
```

### 4.16 Return

```scala
return expr
return
```

### 4.17 For-Comprehension

```scala
for (x <- collection) yield expr
for (x <- collection) body             // imperative
for {
  x <- collection1
  y <- collection2
  if guard
} yield expr
```

### 4.18 While Loop

```scala
while (cond) body
do body while (cond)
```

### 4.19 Assignment

```scala
x = expr
obj.field = expr
```

## 5. Classes

### 5.1 Class Definition

```scala
class Point(val x: Int, val y: Int)
class Point(var x: Int, var y: Int)
class Point(x: Int, y: Int) {         // private constructor params
  def show: String = s"($x, $y)"
}
```

### 5.2 Auxiliary Constructors

```scala
class Point(val x: Int, val y: Int) {
  def this() = this(0, 0)
  def this(x: Int) = this(x, 0)
}
```

### 5.3 Type Parameters

```scala
class Box[T](val content: T)
```

### 5.4 Inheritance

```scala
class Dog(val name: String) extends Animal(name) {
  override def speak: String = "Woof!"
}
```

### 5.5 Abstract Class

```scala
abstract class Shape {
  def area: Double
  def perimeter: Double
}
```

## 6. Traits

```scala
trait Greeter {
  def greet(name: String): String
  def defaultGreet: String = greet("World")
}

trait with Logging {
  def log(msg: String): Unit = println(msg)
}
```

Traits can:
- Declare abstract members
- Provide default implementations
- Be stacked (linearization)

## 7. Objects (Companions)

```scala
object Math {
  def square(x: Int): Int = x * x
}

object Point {
  def origin: Point = new Point(0, 0)
}

case class Point(x: Int, y: Int)
object Point {                              // companion object
  def origin: Point = Point(0, 0)
}
```

## 8. Case Classes

```scala
case class Point(x: Int, y: Int)
```

Auto-generated:
- `apply` factory method on companion
- `unapply` extractor for pattern matching
- `equals`, `hashCode`, `toString`
- `copy` method
- Serializable

## 9. Sealed Traits / Classes

```scala
sealed trait Expr
case class Literal(value: Int) extends Expr
case class Add(left: Expr, right: Expr) extends Expr
case class Mul(left: Expr, right: Expr) extends Expr
```

## 10. Implicit Parameters and Conversions

```scala
implicit val ord: Ordering[Int] = Ordering.Int

def max[A](a: A, b: A)(implicit ord: Ordering[A]): A =
  if (ord.gt(a, b)) a else b

implicit def intToString(x: Int): String = x.toString
```

## 11. Lazy Values

```scala
lazy val x = {
  println("computing")
  42
}
```

## 12. Pattern Matching (Full)

```scala
x match {
  case 0 => "zero"
  case 1 | 2 | 3 => "small"
  case n if n > 10 => "big"
  case List(1, _*) => "starts with 1"
  case head :: tail => s"head=$head"
  case _ => "other"
}
```

## 13. Standard Library

### 13.1 Collections

- `List[A]` — immutable linked list
- `Vector[A]` — immutable indexed sequence
- `Map[K, V]` — immutable hash map
- `Set[A]` — immutable set
- `Option[A]` — `Some(value)` or `None`
- `Either[L, R]` — `Left(value)` or `Right(value)`

### 13.2 Predef Functions

- `println(x: Any): Unit`
- `print(x: Any): Unit`
- `assert(condition: Boolean): Unit`
- `require(condition: Boolean): Unit`
- `throwf(msg: String): Nothing`
- `identity[T](x: T): T`
- `implicitly[T]: T`

### 13.3 Type Members

- `type String = java.lang.String`
- `type RuntimeException = java.lang.RuntimeException`

## 14. Operator Overloading

Methods with symbolic names are operators:

```scala
class Vec(val x: Int, val y: Int) {
  def +(that: Vec): Vec = new Vec(this.x + that.x, this.y + that.y)
  def unary_- : Vec = new Vec(-x, -y)
}
```

## 15. Type Inference

scala supports local type inference:

- `val x = 42` infers `Int`
- `def f(x: Int) = x + 1` infers return type `Int`
- Lambda parameter types may be inferred from context
- **Self-recursive** `def`: the host typechecker accepts a body that calls the same function when a **return type** annotation is present (forward-binding). **Mutual** recursion is not fully supported yet (see **`TODO.md`**).

## 16. Semantics

- Everything is an expression (statements are expressions of type `Unit`)
- Evaluation is eager (strict), except `lazy val` and by-name parameters (`=> T`)
- Parameters are passed by value by default
- `val` is evaluated at definition time
- `var` is reassignable
- Blocks return their last expression
- `Unit` has exactly one value: `()`

## 17. Host tooling (this repository)

Describes the **`scala`** Cargo package binary and library, not Scala language rules.

### 17.1 CLI

| Invocation | Behavior |
|------------|----------|
| `scala file.scala` | Lex, parse, interpret. **No** static type pass unless you opt in (next rows). |
| `scala --check file.scala` | Lex, parse, **`typecheck_program`**, exit (no interpreter). |
| `scala --verify-types file.scala` | Type-check, then interpret (**`typecheck_then_run`**). |
| `scala --repl` | Interactive session; see **§17.3**. |
| `scala --tokens file.scala` | Dump token stream. |
| `scala --ast file.scala` | Pretty-print parsed AST. |
| `scala --version` | Semver from **`Cargo.toml`**. |
| `scala --help` | Short usage. |

### 17.2 Rust API (`src/lib.rs`)

| Function | Purpose |
|----------|---------|
| **`interpret_source`** | Parse + **`Interpreter::run_source`** (no typechecker). |
| **`typecheck_then_run`** | **`typecheck_source`** then fresh interpreter; **`Result<Value, String>`**. |
| **`run_file`** | Used by the CLI for dump / check / run combinations. |

### 17.3 REPL

Built-in commands: `:help`, `:quit` / `:q`, `:reset`. **`:type &lt;expr&gt;`** prints a static type using **`parse_expr`** plus a **fresh prelude-only** **`TypeEnv`** (bindings from earlier REPL lines are **not** visible there yet).

### 17.4 Benchmarks

`cargo bench --bench fib_interp` (Criterion) benchmarks **`typecheck_then_run`** on a small recursive Fibonacci program.
