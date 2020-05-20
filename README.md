# Oxide

An Actor micro-kernel written in Rust

## Overview

The traditional language for writing an operating system is C.
While C is widely portable, and fairly close to Assembly Language, it has a dangerous semantic model.
Rust is a much safer language, which still allows controlled access to "unsafe" mechanisms required for OS development.

Oxide is a memory-safe capability-secure operating system based on the Actor Model of computation.
The [design](docs/design.md) of Oxide is based on a very small asynchronous message-passing micro-kernel.
The three most important design requirements are:
  * Low latency
  * High throughput
  * Provable safety

## Credits

The foundation for this code-base is [Philipp Oppermann's](https://github.com/phil-opp) excellent blog [Writing an OS in Rust](https://os.phil-opp.com/)

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
