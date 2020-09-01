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

# Design

Schrift is a register-based bytecode-driven virtual machine with a traditional `white-switch` dispatch loop.

### Scan

### Parse

### Static analysis

- Compile-time lexical name binding and undefined name resolution

### Schrift IR (SSA)

### Code generation

### Schrift virtual machine

### Runtime

- Efficient composite types backed by a growable array + hashmap combination

### System interfaces

- Raw TCP/IP + FS APIs modeled after POSIX syscalls + userspace polyfills for higher level builtins
- Keep current concurrency model, but support `create()` / `join()` / `send()` threading with message passing (outlined in Ink spec in main repository)
- `exit()` syscall, better APIs around file descriptors so we can do things like connect pipes between child processes of `exec()`
- In general, we should try to reduce number of syscalls the runtime needs to make, and expose APIs for Ink programs to also minimize their syscall counts
- Syscalls should expose a generic interface for interfacing with memory-mapped hardware (?)
    - I want to be able to run Ink on a microcontroller or Raspberry Pi and control hardware peripherals like servos and LEDs directly

## Schrift bytecode

The VM's bytecode is a flattened, optimized representation of an Ink program. The main goal of the bytecode format is

1. take advantage of data locality to improve performance
2. Enable low level optimizations and code reduction

## Garbage collection

- Use ARC instead of tracing GC, different perf characteristics, better memory efficiency, better suited to Rust
    - Is `Rc<T>` appropriate here at all? Or do we need our own ARC impl
    - Example and blog: https://github.com/Manishearth/rust-gc

