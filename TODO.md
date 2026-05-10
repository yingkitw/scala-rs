# scala-rs Implementation TODO

## Phase 1: Lexer / Tokenizer
- [x] Token type definitions (keywords, operators, literals, identifiers, delimiters)
- [x] Lexer that converts source text to token stream
- [x] Integer / Long / Double / Float literal parsing
- [x] String literal parsing (including escape sequences)
- [x] Character literal parsing
- [x] Multi-line string literals (triple-quoted)
- [x] String interpolation (`s"..."`, `f"..."`, `raw"..."`)
- [x] Comment handling (single-line `//` and block `/* */`)
- [x] Operator and identifier recognition
- [x] Source location tracking (line, column)
- [x] Error reporting with source positions

## Phase 2: AST (Abstract Syntax Tree)
- [x] Source span tracking on all nodes
- [x] Expression nodes: literals, binary/unary ops, if/match/try/throw, block, lambda, tuple, assign
- [x] Declaration nodes: val, var, def, class, trait, object, case class, type alias
- [x] Pattern nodes: wildcard, variable, literal, constructor, tuple, typed, alternative
- [x] Type annotation nodes
- [x] Import and package nodes
- [x] For-comprehension nodes
- [x] New expression and method call nodes
- [x] AST pretty-printing / debug formatting

## Phase 3: Parser
- [x] Recursive-descent parser with precedence climbing for expressions
- [x] Top-level declarations (package, import, val/var, def, class, trait, object)
- [x] Expression parsing (all precedence levels)
- [x] Pattern parsing for match expressions and val/def destructuring
- [x] Type parsing (simple, parameterized, function, tuple, compound)
- [x] String interpolation parsing
- [x] For-comprehension parsing (with yield and imperative)
- [x] Class body parsing (fields, methods, constructors)
- [x] Error recovery and reporting
- [x] Operator precedence table matching Scala spec

## Phase 4: Type System & Type Checker
- [x] Type representation (primitive, function, parameterized, tuple, etc.)
- [x] Type environment with scoping
- [x] Type checking for all expression forms
- [x] Local type inference for val/var/def
- [x] Lambda type inference from context
- [x] Subtyping rules (Any > AnyRef > ... > Nothing)
- [x] Generic type parameter handling
- [x] Trait and class type checking
- [x] Pattern exhaustiveness checking (sealed traits)
- [x] Type error reporting

## Phase 5: Interpreter / Evaluator
- [x] Runtime value representation
- [x] Environment with lexical scoping and closures
- [x] Expression evaluation (arithmetic, comparison, boolean, string)
- [x] Block evaluation with nested scopes
- [x] Function application (including closures)
- [x] Class instantiation
- [x] Trait mixin composition
- [x] Field access and method dispatch
- [x] Pattern matching evaluation
- [x] Control flow: if/else, while, for, match, try/catch/finally
- [x] String interpolation evaluation
- [x] Tail-call optimization awareness
- [x] Exception handling

## Phase 6: REPL
- [x] Interactive read-eval-print loop
- [x] Multi-line input detection (bracket matching)
- [x] Expression and declaration handling
- [x] Persistent environment across inputs
- [x] Type display for expressions
- [x] Tab completion (basic)
- [x] History support
- [x] Ctrl-C / Ctrl-D handling

## Phase 7: Standard Library
- [x] Predef: println, print, assert, require, identity
- [x] List: map, filter, foldLeft, foldRight, head, tail, isEmpty, length, ::, ++, flatMap, foreach
- [x] Option: map, flatMap, filter, get, getOrElse, isDefined, isEmpty, orElse
- [x] Map: apply, get, +, -, updated, contains, keys, values, foreach
- [x] String methods: length, substring, split, trim, toUpperCase, toLowerCase, replace, contains
- [x] Numeric conversions: toInt, toLong, toDouble, toFloat
- [x] Tuple access: _1, _2, ... _N
- [x] Range: to, until, by
- [x] Math functions: abs, max, min, pow, sqrt, Pi, E

## Phase 8: CLI & Integration
- [x] File execution mode
- [x] REPL mode
- [x] Type-check-only mode (--check)
- [x] AST dump mode (--ast)
- [x] Token dump mode (--tokens)
- [x] Error formatting with source context

## Phase 9: Testing & Polish
- [x] Unit tests for lexer
- [x] Unit tests for parser
- [x] Unit tests for type checker
- [x] Unit tests for interpreter
- [x] Integration tests with Scala source files
- [x] Error message quality
- [x] Performance benchmarks
