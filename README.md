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

APIs

- Raw TCP/IP + FS APIs modeled after POSIX syscalls + userspace polyfills for higher level builtins
- Keep current concurrency model, but support `create()` / `join()` / `send()` threading with message passing (outlined in Ink spec in main repository)

