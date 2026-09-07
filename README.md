# Baton - Custom Compiler targeting ARM64 Assembly 

[![CI/CD Pipeline](https://github.com/Ridju/baton/actions/workflows/ci.yml/badge.svg)](https://github.com/Ridju/baton/actions)
![Language](https://img.shields.io/badge/language-rust-blue)
![Target](https://img.shields.io/badge/target-ARM64_Assembly-orange)
![Tests](https://img.shields.io/badge/tests-passing-brightgreen)



Custom compiler for a custom language (`.ba`) inspired by C, written in **Rust**. The compiler translates source code directly into **ARM64 assembly** (target: Apple Silicon) and generates a native, executable binary.

## Architecture & Core Components

* **Low-Level Lexer:** Hand-written scanner operating on byte slices for high-performance tokenization
* **Recursive Descent Parser:** Clean, modular syntax analysis translating source input into a typed Abstract Syntax Tree (AST)
* **Semantic Analysis:** Handles symbol table management, lexical scoping, and type-checking prior to emission
* **ARM64 Code Generation:** Translates the validated AST directly to native ARM64 assembly
* **Comprehensive Testing Suite:** Exhaustive unit tests alongside automated **End-to-End (E2E) integration tests**
* **CI/CD (GitHub Workflows)**: Automated GitHub Actions to run tests automatically when pushing to the main branch


## Supported Features

* **Rich Developer Diagnostics:** Custom error reporting inspired by modern toolchains (like Rust/Cargo), providing precise line/column indicators and source code snippets for syntax and semantic errors:

**Source Code (`invalid.ba`):**
```c
int a = @10;
```

**Output:**
```text
[ERROR]: Unexpected character: '@'
  --> line 1:10
   |
 1 | int a = @10;
   |         ^
   |
```

* **Strong Typing:** Support primitive types & composite types 
```c
//primitive types
int x = 42;
bool active = true;
float pi = 3.14;
string msg = "Hello World";

//composite types
struct Point {
  float x;
  float y;
};
float [] nums;
```
* **Control Flow:** Support for conditional branching and while loops
```c
int x = 10;

//conditional branching
if (x > 5) {
  print("Greater than 5");
} else {
  print("Less or equal 5");
}

//loops
while (x > 0 ) {
  x = x - 1;
}
```
* **Functions & Recursion:** Support for functions with parameters, return values and deep recursion 
```c
int factorial(int n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
```

* **Native Print Function:** Built-in `print` function to output values to the command line
```c
int main() {
  print("Hello World");
}
```
* **Comments:** Support for clean code documentation via comments 
```c
// Single-line comment 
int x = 42; // Inline comments
```

## E2E Test Suite (`.ba`)

Set of end-to-end integration test programs used to verify compiler correctness across multiple compilation stages:

* **`./examples/hello_welt.ba`**
  * Demonstrates basic program structure, string literal handling, and the native `print` function
* **`./examples/power.ba`**
  * Tests mathematical expressions, parameters, and recursive function calls
* **`./examples/fibonacci.ba`**
  * Validates function declarations, return values, and deep recursion combined with conditional logic
* **`./examples/ggt.ba`**
  * Exercises `while` loops, nested `if` statements, variable reassignment, and the Euclidean algorithm (via subtraction)
* **`./examples/foobar.ba`**
  * Acts as a classic integration test exercising control flow, logic, and multiple print statements
* **`./examples/point.ba`**
  * Demonstrates custom `struct` definition, member field assignment, and arithmetic operations on struct properties
* **`./examples/sum.ba`**
  * Validates array declaration, index-based assignment, traversal via `while` loops, and floating-point accumulation logic


## Getting Started & Usage

### Prerequisites & Development Environment (DevContainer)
This project features a fully configured **DevContainer**. If you are using VS Code, simply open the repository in the container, and all dependencies—including **Rust (Cargo)**, **Clang/`cc`**, and the required ARM64 build environment—will be automatically provided out of the box.

Alternatively, you can run it locally with:
* **Rust & Cargo** (latest stable release)
* **Clang / `cc`** (compatible C compiler/linker)

### Quickstart

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/Ridju/baton
    cd baton
    ```

2.  **Build the compiler in release mode:**
    ```bash
    cargo build --release
    ```

### Compiling & Running Examples

Run Built-in `.ba` test files (executables are automatically routed to the output/ directory):

```bash
# Compile with a custom output binary name
cargo run --release -- examples/fibonacci.ba -o fib

# Compile and keep the intermediate ARM64 assembly file (using -k or --keep-asm)
cargo run --release -- examples/fibonacci.ba -o fib -k

# Execute the resulting binary directly
./output/fib
```