// Classes, objects, and pattern matching
class Point(val x: Int, val y: Int)

val p1 = new Point(3, 4)
val p2 = new Point(0, 0)

println("Classes")
println("p1.x = " + p1.x)
println("p1.y = " + p1.y)

// Object with methods
object MathUtils {
  def max(a: Int, b: Int): Int = if (a > b) a else b
  def min(a: Int, b: Int): Int = if (a < b) a else b
  val pi = 31415
}

println("MathUtils.max(10, 20) = " + MathUtils.max(10, 20))
println("MathUtils.min(10, 20) = " + MathUtils.min(10, 20))
println("MathUtils.pi = " + MathUtils.pi)

// Match expression
def describe(n: Int): String = n match {
  case 0 => "zero"
  case 1 => "one"
  case _ => "other"
}

println("Match:")
println("describe(0) = " + describe(0))
println("describe(1) = " + describe(1))
println("describe(42) = " + describe(42))

// Nested match
def classify(n: Int): String = {
  if (n < 0) "negative"
  else if (n == 0) "zero"
  else if (n <= 10) "small"
  else "large"
}

println("classify(-5) = " + classify(-5))
println("classify(0) = " + classify(0))
println("classify(5) = " + classify(5))
println("classify(100) = " + classify(100))
