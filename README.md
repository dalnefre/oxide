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

## Building

You need a nightly Rust compiler. First you need to install the `cargo-xbuild` and `bootimage` tools:

```
cargo install cargo-xbuild bootimage
```

Then you can build the project by running:

```
cargo xbuild
```

To create a bootable disk image, run:

```
cargo bootimage
```

This creates a bootable disk image in the `target/x86_64-blog_os/debug` directory.

## Running

You can run the disk image in [QEMU] through:

[QEMU]: https://www.qemu.org/

```
cargo xrun
```

Of course [QEMU] needs to be installed for this.

You can run [QEMU] directly like this:

```
qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin
```

You can also write the image to an USB stick for booting it on a real machine. On Linux, the command for this is:

```
dd if=target/x86_64-blog_os/debug/bootimage-blog_os.bin of=/dev/sdX && sync
```

Where `sdX` is the device name of your USB stick. **Be careful** to choose the correct device name, because everything on that device is overwritten.

## Testing

To run the unit and integration tests, execute `cargo xtest`.

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Credits

The foundation for this code-base is [Philipp Oppermann's](https://github.com/phil-opp) excellent blog [Writing an OS in Rust](https://os.phil-opp.com/)
