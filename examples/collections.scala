// List operations and for-comprehensions
val nums = List(10, 20, 30, 40, 50)

println("List Operations")
println("nums = " + nums.mkString(", "))
println("head = " + nums.head)
println("length = " + nums.length)
println("isEmpty = " + nums.isEmpty)
println("contains(30) = " + nums.contains(30))
println("indexOf(30) = " + nums.indexOf(30))

// Transformations
val doubled = nums.map((x) => x * 2)
println("doubled = " + doubled.mkString(", "))

val big = nums.filter((x) => x > 25)
println("big = " + big.mkString(", "))

val sum = nums.reduce((a, b) => a + b)
println("sum = " + sum)

val reversed = nums.reverse
println("reversed = " + reversed.mkString(", "))

val sorted = List(5, 3, 1, 4, 2).sorted
println("sorted = " + sorted.mkString(", "))

// For comprehensions
val squares = for (x <- List(1, 2, 3, 4, 5)) yield x * x
println("squares = " + squares.mkString(", "))

val evenSquares = for {
  x <- List(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
  if x % 2 == 0
} yield x * x
println("evenSquares = " + evenSquares.mkString(", "))

// Nested for
val pairs = for {
  x <- List(1, 2)
  y <- List(10, 20)
} yield x + y
println("pairs = " + pairs.mkString(", "))

// String operations
val greeting = "Hello, Scala!"
println("String Operations")
println("greeting = " + greeting)
println("length = " + greeting.length)
println("toUpperCase = " + greeting.toUpperCase)
println("contains(\"Scala\") = " + greeting.contains("Scala"))
