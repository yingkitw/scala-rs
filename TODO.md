# scala Implementation TODO

## Phase 1: Lexer / Tokenizer
- [x] Token type definitions (keywords, operators, literals, identifiers, delimiters)
- [x] Lexer that converts source text to a token stream
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
- [x] Source span tracking on AST nodes where applicable
- [x] Expression nodes: literals, binary/unary ops, if/match/try/throw, block, lambda, tuple, assign
- [x] Declaration nodes: val, var, def, class, trait, object, case class, type alias
- [x] Pattern nodes: wildcard, variable, literal, constructor, tuple, typed, alternative
- [x] Type annotation nodes
- [x] Import and package nodes
- [x] For-comprehension nodes
- [x] New expression and method call nodes
- [x] AST pretty-print / debug formatting

## Phase 3: Parser
- [x] Recursive-descent parser with precedence climbing for expressions
- [x] Top-level declarations (package, import, val/var, def, class, trait, object)
- [x] Expression parsing (major precedence levels)
- [x] Pattern parsing for match expressions and bindings
- [x] Type parsing (simple, parameterized, function, tuple, compound)
- [x] String interpolation parsing
- [x] For-comprehension parsing (with yield and imperative)
- [x] Class body parsing (fields, methods, constructors)
- [x] Error reporting at parse failures
- [ ] Full operator precedence parity with Scala 3 spec (`:` family, assignments in wider positions, etc.)

## Phase 4: Type System & Type Checker
- [x] Type representation (`ty.rs`) with function, tuple, named, and built-ins
- [x] Type environment with lexical scoping
- [x] Type checking across major expression shapes
- [x] Parameter and return annotations on `def`
- [x] Local inference for bindings where implemented
- [x] Subtyping / lub rules (within current feature set)
- [x] Type error diagnostics with spans
- [x] **Forward binding for self-recursive `def`** (`def fact(n: Int): Int = ... fact(n - 1) ...`)
- [x] **Top-level `object` name visible after decl** (`object O { ... }; O.method(...)`) — module typeapproximation
- [ ] Mutual recursion (`def f = g()`, `def g = f()`): forward declare one pass or SCC
- [ ] Stricter arithmetic / relational checks (narrow `Ty::Any` fallout from sloppy binops)
- [ ] Narrow `val` / `var` mismatches (`val x: Int = "oops"`) with real subtyping
- [ ] `object`-member method signatures exposed on the module type (not only `Ty::Named` stubs)
- [ ] Sealed exhaustive match checking beyond current patterns

## Phase 5: Interpreter / Evaluator
- [x] Runtime value representation
- [x] Environment with lexical scoping and closures
- [x] Expression evaluation across supported forms
- [x] Class instantiation and field/method dispatch
- [x] Pattern matching evaluation
- [x] Control flow: if/else, while, for, match, try/catch (as implemented)
- [x] Interpreter unit tests (`src/interpreter.rs`)
- [ ] Optional **type-check before run** wired as default CLI for files (currently opt-in `--verify-types` to preserve old behavior)

## Phase 6: REPL
- [x] Read–eval–print loop (rustyline or stdin fallback)
- [x] Multi-line bracket heuristic
- [x] Persistent environment across submissions
- [x] `:help`, `:quit`, `:reset`
- [x] **`:type <expr>` (prelude / builtins scope only)** — see note in help text
- [ ] `:type` that reflects live REPL bindings (share type state with evaluator)
- [ ] Tab completion / history parity on basic stdin mode

## Phase 7: Standard Library
- [x] Core `println`, `print`, assertions, collections surface in `stdlib.rs`
- [x] `List`, `Option`, `Map`, string helpers, conversions, numeric helpers where wired
- [ ] Broader Scala `Predef` / `collections` fidelity
- [ ] User-defined `@main` entry points

## Phase 8: CLI & Embedding API
- [x] `--repl`, `--check`, `--tokens`, `--ast`
- [x] **Version strings from Cargo** (`env!("CARGO_PKG_VERSION")`)
- [x] **`scala::typecheck_then_run`** — lex, parse, typecheck, interpret; returns `Result<Value, String>`
- [x] **`scala::interpret_source`** — run without static pass (matching `Interpreter::run_source`)
- [x] **`--verify-types <file.scala>`** — typecheck ok, then run file
- [ ] Single combined pipeline that parses once and shares AST between check and interpreter (avoid triple lex/parse on `--verify-types`)

## Phase 9: Testing & Bench
- [x] Unit tests: lexer, parser, interpreter modules
- [x] Integration tests (`tests/integration_tests.rs`)
- [x] **Compile-vs-run gates** (`tests/compile_and_interpret.rs`)
- [x] **Criterion bench** skeleton (`cargo bench`) for interpret throughput
- [ ] Golden/fixture Scala programs under `examples/` exercised in CI

## Phase 10: Docs & Repo Hygiene
- [x] **`README.md`**, **`ARCHITECTURE.md`**, and **`SPEC.md`** aligned with current CLI, library API (`typecheck_then_run`, `interpret_source`), `--verify-types`, REPL `:type` limits, benches, default no-typecheck file run
- [ ] Repeat alignment after future language / CLI churn
- [ ] Expand `docs/` beyond `index.html` if a static site is needed
