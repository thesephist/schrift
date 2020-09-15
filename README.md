# Schrift 🚄

**Schrift** is an experimental runtime for the [Ink](https://github.com/thesephist/ink) programming language, focused on performance and debugging experience.

Schrift is currently **under development**. Most parts of the runtime are not working yet.

## Motivation

I first wrote the Ink language and its Go-based interpreter as a toy project to learn about parsers and interpreters. Because of that provenance, the original Go interpreter has lots of shortcomings, especially in performance and runtime instrumentation capabilities.

Schrift is my second attempt at an Ink interpreter, focused on performance and debugging, and better architecture in general. It's not designed to be a complete replacement, but if Schrift is successful, I think it can be better than the Go-based interpreter in almost every metric.

## Goals

- Performance, both execution speed and memory efficiency. 
- Correctness, specifically 100% compatibility with the existing interpreter.
- Great error messages and stack traces.
- Support for opt-in profiling and tracing at runtime.
- Better designed system interfaces and APIs for things like `exec()` and filesystem control.

## Design

Schrift is a register-based bytecode virtual machine with a traditional `white-switch` dispatch loop.

### Scan & parse

Schrift contains a parser for the full Ink language grammar, including comments.

### Static analysis

Static analysis in Schrift performs at least the following, for code generation and optimization.

- Variable declaration annotation
- Escape analysis
- Expression normalization for easier code generation

### Code generation and bytecode format

The VM's bytecode is a flattened, optimized representation of an Ink program. The main goal of the bytecode format is

1. Take advantage of data locality to improve runtime performance.
2. Enable instruction-level pipelining and parallelism at runtime, as much as possible.
3. Provide a good format for code optimization

Schrift's bytecode is register-based and designed to be an optimized single static assignment (SSA) form of the Ink AST. Each function and expression list in Ink is compiled to a separate contiguous block of bytecode, called `Block`, to allow for incremental compilation and replacements of parts of a program during interactive evaluation of a program.

### Optimizations

On the bytecode, Schrift performs at least the following optimizations.

- Constant propagation
- Common subexpression elimination
- Dead branch/code elimination
- Function call inlining
- Tail call elimination (unrolling tail recursion into loops)

### Schrift virtual machine

The VM design is to be done, but will include the following.

- Composite values are backed by both a growable array and a hashmap, a la JavaScript and Lua.

### Runtime and garbage collection

Initially, Schrift will use automatic reference counting for memory management. This is because ARC is:

- More memory-efficient
- Lower latency than a tracing mark-and-sweep GC
- Better suited to Rust's ownership model

Some open questions about ARC in Rust and Schrift:

- Is `Rc<T>`/`Arc<T>` appropriate here at all? Or do we need our own implementation or wrapper?
- Example and blog: https://github.com/Manishearth/rust-gc
- When does Ink leak memory with ARC, and how can we allow control or workarounds in those situations?

### Builtins and system interfaces

- Raw TCP/IP + FS APIs modeled after POSIX syscalls + userspace polyfills for higher level builtins
- In general, we should try to reduce number of syscalls the runtime needs to make, and expose APIs for Ink programs to also minimize their syscall counts
- Syscalls should expose a generic interface for interfacing with memory-mapped hardware (?)
    - I want to be able to run Ink on a microcontroller or Raspberry Pi and control hardware peripherals like servos and LEDs directly
- `exit()` syscall, better APIs around file descriptors so we can do things like connect pipes between child processes of `exec()`
- Keep current concurrency model, but best-effort support for `create()` / `join()` / `send()` threading with message passing (outlined in Ink spec in main repository)

