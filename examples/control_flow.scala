// Control flow: if/else, while, blocks, tuples
println("Control Flow")

// If/else as expression
val x = 42
val label = if (x > 50) "big" else if (x > 20) "medium" else "small"
println("x = " + x + ", label = " + label)

// Block expression
val result = {
  val a = 10
  val b = 20
  a + b
}
println("block result = " + result)

// While loop
var i = 0
var total = 0
while (i < 10) {
  total = total + i
  i = i + 1
}
println("sum 0..9 = " + total)

// Tuples
val point = (10, 20)
println("tuple = " + point)

// Nested blocks with shadowing
val outer = 1
val inner = {
  val outer = 2
  outer + 10
}
println("outer = " + outer + ", inner = " + inner)

// Boolean logic
val a = true
val b = false
println("true && false = " + (a && b))
println("true || false = " + (a || b))
println("!true = " + !a)

// Comparison chaining
val cmp1 = 1 < 2 && 2 < 3 && 3 < 4
println("1 < 2 < 3 < 4 = " + cmp1)
