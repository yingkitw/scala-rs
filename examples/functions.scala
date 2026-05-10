// Functions, recursion, and higher-order functions
def square(x: Int): Int = x * x
def cube(x: Int): Int = x * x * x
def abs(x: Int): Int = if (x < 0) -x else x

println("Functions")
println("square(5) = " + square(5))
println("cube(3) = " + cube(3))
println("abs(-42) = " + abs(-42))
println("abs(42) = " + abs(42))

// Recursion
def factorial(n: Int): Int = if (n <= 1) 1 else n * factorial(n - 1)
def fib(n: Int): Int = if (n <= 1) n else fib(n - 1) + fib(n - 2)

println("factorial(10) = " + factorial(10))
println("fib(10) = " + fib(10))

// Higher-order functions
def apply(f: Int => Int, x: Int): Int = f(x)
val result = apply(square, 7)
println("apply(square, 7) = " + result)

// Lambda expressions
val doubled = List(1, 2, 3, 4, 5).map((x) => x * 2)
println("doubled = " + doubled.mkString(", "))

val evens = List(1, 2, 3, 4, 5, 6, 7, 8, 9, 10).filter((x) => x % 2 == 0)
println("evens = " + evens.mkString(", "))
