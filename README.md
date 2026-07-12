# Aglang
![Aglang logo](./assets/aglang-logo.avif)

Aglang is a **stack-based, esoteric language** made to be simple, _like Assembly_.

This repository contains the CLI with an interpreter and a compiler.

_If you're looking for the website's repository, you can find it [here](https://github.com/0xF1dev/aglang-website)._

## Introduction

Aglang is **relatively simple** compared to other esolangs, like **Brainf*ck**. In it, the programmer has three things
available: **two 8-bit registers** and a **stack**.

With these, almost **anything is possible!** In the `examples/` directory, you can find the following:

1. **Hello, World:** A program that outputs "Hello, World!" to the terminal;
2. **Fibonacci:** A program that computes the first 13 numbers in the Fibonacci sequence, the maximum permitted by the
   8-bit integer limit.
3. **Mirror:** A program that takes the users input and prints it back.

## Usage

First, clone this repository (`git clone https://github.com/0xF1dev/aglang.git && cd aglang`) and install the CLI with:
```shell
cargo install --path .
```
or use the precompiled executable in the Releases tab.

---

The CLI has two commands, `run` and `build`. Here's how you can use them:

### Run with the interpreter
```shell
aglang run <FILE>
```

### Build into an executable
```shell
aglang build <FILE> -o <OUTPUT>
```
_(note: compilation currently only supports x86-64 Linux; if your machine isn't supported, the compiler will give you an error)_

_(note 2: to build a file, you need to have GCC installed)_

## Quick reference

The following table contains all the valid tokens and their use.

|    Token    | Description             | Note                                                                                                                                                                                                                                                       |
|:-----------:|-------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|     `;`     | **Statement delimiter** | The semicolon is placed at the end of a statement to declare its end.                                                                                                                                                                                      |
| `0` and `1` | **Binary literals**     | 8-bit binary numbers can be used as number literals.                                                                                                                                                                                                       |
|     `'`     | **Register 1**          | The **Register 1** is an 8-bit register used to store a value.                                                                                                                                                                                             |
|    `''`     | **Register 2**          | The **Register 2** is an 8-bit register used to store a value.                                                                                                                                                                                             |
|     `:`     | **Stack**               | The **Stack** is a _LIFO_ array of 8-bit values; only the top value in it can be read. While it can be directly read via the _copy_ operator, the top value cannot be directly written, it can only be pushed and popped.                                  |
|     `>`     | **Copy**                | The **Copy** operator is used to copy or insert values from a source to a destination                                                                                                                                                                      |
|     `!`     | **Remove**              | The **Remove** operator is used to pop (in the case of the stack) or clear (in the case of a register) a value.                                                                                                                                            |
|    `\|`     | **Input**               | The **Input** operator is used to wait a value from the `StdIn` and insert it in the stack in reverse order and delimited at the bottom by a 0.                                                                                                            |
|     `+`     | **Sum**                 | The **Sum** operator is used to sum 2 values. The result gets stored in the first argument. In the case of an overflow, the result wraps around to 0.                                                                                                      |
|     `-`     | **Subtraction**         | The **Subtraction** operator is used to subtract 2 values. The result gets stored in the first argument. In the case of an underflow, the result wraps around to 0.                                                                                        |
|     `*`     | **Multiplication**      | The **Multiplication** operator is used to multiply 2 values. The result gets stored in the first argument. In the case of an overflow, the result wraps around to 0.                                                                                      |
|     `/`     | **Division**            | The **Division** operator is used to divide 2 values and get the quotient. The result gets stored in the first argument.                                                                                                                                   |
|     `%`     | **Remainder**           | The **Remainder** operator is used to get the remainder of the division of the 2 arguments. The result gets stored in the first argument.                                                                                                                  |
|     `\`     | **StdOut**              | The **StdOut** can be used as the destination of a _copy_ operation to print the ASCII character of the source to the terminal. When a `#` character is appended to it (`\#`), the value gets printed as its decimal value instead of the ASCII character. |
| `[` and `]` | **Loop**                | A **Loop** runs until, at the end of it, the top value in the stack is `0`.                                                                                                                                                                                |
