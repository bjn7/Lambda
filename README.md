# Lambda Programming Language

Lambda is an esoteric programming language inspired (not based on) by lambda calculus.

## How to Use It

First, you need the interpreter binary. It's pretty easy to build this since it has a relatively simple codebase.

1. Make sure you have Rust installed.
2. Run `cargo build --release`.
3. The target binary will be located at `target/release/lamda`.

Make sure to add this binary path (`target/release`) to your environment variables.

### Example: "Hello World!" in Lambda

Here’s a sample code to print "Hello World!":

```lamda
(λascii. ascii) 72 (λascii. ascii) 101 (λascii. ascii) 108 (λascii. ascii) 108 (λascii. ascii) 111
(λascii. ascii) 44
(λascii. ascii) 32 (λascii. ascii) 87 (λascii. ascii) 111 (λascii. ascii) 114 (λascii. ascii) 108 (λascii. ascii) 100 (λascii. ascii) 33
(λascii. ascii) 10
````

1. Create a file called `main.lamda`.
2. Paste the code into this file.
3. Run it via `lamda main.lamda`.

---

## Syntax

### Binding

**Syntax**: `<variable> = <expression>`

An expression can be anything except for a binding. It can be an abstraction, an application, a binary operation, or a numeric literal.

**Example**:

```lamda
x = 5
y = λv. v + 1
```

### Abstraction

**Syntax**: `λ<parameter>.<expression>`

This is called an abstraction. For example:

```lamda
λx. x + 1
```

Abstractions form the foundation of applications. Lambda has five built-in abstractions.

#### Built-in Abstractions

* **λascii**: Takes an ASCII value in decimal form (0 to 255) and prints the corresponding ASCII character.
* **λprint**: Takes a numerical value and prints it as is.
* **λinput**: Accepts either 0 or 1 as an argument:

  * If 0 is provided, it accepts a single character (including numbers ascii) and returns its corresponding ASCII decimal value.
  * If 1 is provided, it accepts a valid numeric value.
* **λtime**: Returns the current system time in Unix Epoch.
* **λsleep**: Pauses execution for a given number of milliseconds.

### Application

**Syntax**: `(<Abstraction>) <parameter value>`

An application occurs when a value is passed to an abstraction. The abstraction applies the parameter and returns the result.

**Example**:

```lamda
(λv. v + 1) 10
```

Here, the value `10` is passed to the abstraction as `v`, and the result is `v + 1`, which is `11`.

You can also bind the result to a variable:

```lamda
z = (λv. v + 10) 10
```

Alternatively, you can pass the result directly to another abstraction:

```lamda
(λprint. print) (λv. v + 10) 10
```

Or:

```lamda
(λprint. (λv. v + 10) 10) 0
```

Both do the same thing, printing `20`. The `0` is just passed as a parameter to preserve strucutre, omitting the `0` will result in an error.

### Recursion

**Syntax**: `λ<parameter>.𝑓(<expression>)`

Recursion in Lambda calls itself until an argument of `0` is provided (i.e., `λn.𝑓(0)`). This sends a "HALT" signal, and if any function receives this signal, it will stop executing.

**Example**:

```lamda
(λprint. 𝑓(print-1)) 10
(λascii. ascii) 10
```

The above code will print numbers from 9 to 0, followed by a LF, i.e., '\n'.

```lamda
(λx. x) λn.𝑓(0)
```

Here, `𝑓(0)` returns the "HALT" signal, so `x` will not execute. You can also capture this with bindings:

```lamda
halt = ( λn.𝑓(0) ) 0
(λinput. input) halt
```

In this example, the `input` function will not execute because it receives the "HALT" signal.