# Baton - Custom Compiler in Rust (ARM64)

A custom compiler for a C-like programming language, written in **Rust**. The compiler translates source code directly into **ARM64 assembly** (target: Apple Silicon) and generates a native, executable binary.

## Supported Features

* **Strong Typing:** Supports `int`, `bool`, `float`, `string`, as well as composite types like `structs` and `arrays`.
* **Lexer & Parser:** **Recursive descent parser** with AST generation and error handling.
* **Control Flow:** `if`/`else` conditionals and `while` loops.
* **Functions & Recursion:** Functions with parameters, return values, and deep recursion support.
* **Native Code Generation:** Emits ARM64 assembly code and leverages `cc` (Clang) to link the final executable.

---

## Example Programs (`/examples`)
* **`hello_welt.ba`**
  * Basic program structure, string literal handling, and the print statement.
* **`potenz.ba`**
  * Mathematical expressions, recursive function calls, and multiplication.
* **`fibonacci.ba`**
  * Function declarations, parameters, return values, and deep recursion combined with conditional logic.
* **`ggt.ba`**
  * `while` loops, nested `if` statements, variable reassignment, and the Euclidean algorithm (via subtraction).
* **`foobar.ba`**
  * Implementation of famous foobar programm.


---

## Usage & Execution

Prerequisites: **Rust (Cargo)** and an ARM64-compatible C compiler (`cc` / `Clang` on macOS).

```bash
# Clone or open the repository
cd your-compiler-repo

# Build the compiler in release mode
cargo build --release

# Compile and run an example program
cargo run --release examples/fibonacci.ba -o fib_prog
./output/fib_prog
```