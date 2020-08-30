# Schrift 🚄

A more experimental runtime for Ink

## Ideas and brainstorming

Goals

- Speed
- Correctness and 100% backwards compatibility with the canonical interpreter. Best-effort forward compat
- Broader, more experimental feature set geared for lower level control and performance

Design & requirements

- Support profiling, proper error traces
- Compile-time lexical name binding and undefined name resolution
- Bytecode compiler + VM
- Efficient composite types backed by a growable array + hashmap combination
- Use ARC instead of tracing GC, different perf characteristics, better memory efficiency, better suited to Rust
    - Is `Rc<T>` appropriate here at all? Or do we need our own ARC impl
    - Example and blog: https://github.com/Manishearth/rust-gc

APIs

- Raw TCP/IP + FS APIs modeled after POSIX syscalls + userspace polyfills for higher level builtins
- Keep current concurrency model, but support `create()` / `join()` / `send()` threading with message passing (outlined in Ink spec in main repository)
- `exit()` syscall, better APIs around file descriptors so we can do things like connect pipes between child processes of `exec()`
- In general, we should try to reduce number of syscalls the runtime needs to make, and expose APIs for Ink programs to also minimize their syscall counts
- Syscalls should expose a generic interface for interfacing with memory-mapped hardware (?)
    - I want to be able to run Ink on a microcontroller or Raspberry Pi and control hardware peripherals like servos and LEDs directly

