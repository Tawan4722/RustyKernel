# Rust x86_64 UEFI Research Kernel (Limine)

This is RustKernel Expirement By openAI codex.

This repository contains a kernel-first research OS baseline:

- Limine boot protocol on `x86_64` + UEFI
- Kernel bring-up with GDT/IDT/PIC/timer
- Boot memory parsing, frame allocation, and heap setup
- Round-robin kernel task scheduler
- Minimal syscall ABI surface (`write`, `exit`, `yield`)
- One user-mode smoke ELF module and ring3 transition path

## Credits

- Project owner and lead maintainer: **Tawan4722** ([RustyKernel](https://github.com/Tawan4722/RustyKernel))
- Boot protocol/runtime foundations used by this project:
  - Limine boot protocol
  - Rust OS ecosystem crates (`x86_64`, `pic8259`, `uart_16550`, `linked_list_allocator`, `xmas-elf`)

## How This Works

1. Limine loads `kernel.elf` and module `user-smoke.elf` via UEFI.
2. Kernel entry (`_start`) initializes logging, reads Limine boot responses, then brings up GDT/IDT/PIC and memory.
3. Kernel enables a simple frame allocator + heap and starts a preemptive timer-driven scheduler.
4. Syscall trap path (`int 0x80`) is wired with a minimal POSIX-like syscall table (`write`, `exit`, `yield`).
5. A small ring3 smoke ELF is mapped and entered; it traps twice to validate user->kernel->user control flow.

## Why It Is Designed This Way

- **Kernel-first baseline**: fastest route to a reliable bring-up loop before adding full process model/filesystem/network.
- **Limine + UEFI**: avoids writing a custom bootloader and keeps hardware/QEMU boot flow practical.
- **SMP-ready structure, BSP execution now**: keeps v1 stable while preventing architecture dead-ends.
- **Narrow syscall ABI v1**: validates privilege transitions early without overcommitting to unstable interfaces.
- **Module-based user smoke app**: end-to-end ring3 proof without requiring full userland tooling yet.

## Build and Run

```powershell
cargo boot-build
cargo run-qemu
```

Debug QEMU (GDB stub at `:1234`):

```powershell
cargo run-qemu-debug
```

Prepare a hardware handoff artifact:

```powershell
cargo run-hardware-artifact
```

`dist/kernel-uefi.zip` is the UEFI payload for a FAT partition (`EFI/BOOT/BOOTX64.EFI`, `limine.conf`, `kernel.elf`, `user-smoke.elf`).

## Prerequisites

- Rust stable toolchain
- `qemu-system-x86_64`
- `git` (for Limine binary checkout)
- Optional: `OVMF_PATH` env var if QEMU cannot auto-locate OVMF firmware

## Syscall ABI v1

Calling convention target: x86_64 user/kernel boundary (POSIX-like numbering).

| Number | Name   | Status in v1 |
|--------|--------|---------------|
| `1`    | write  | Implemented for smoke path |
| `2`    | exit   | Implemented for smoke path |
| `3`    | yield  | Reserved for scheduler integration |

Return contract: `>=0` success, `<0` negative errno (`-22` invalid arg, `-38` not implemented).

## License

This repository currently includes an `AGPL-3.0` license file.

For a recommendation and tradeoffs (AGPL vs permissive dual licensing), see:
- [docs/license-recommendation.md](docs/license-recommendation.md)

## Graphs

- [Architecture and boot graphs](docs/architecture-graph.md)
- [Architecture picture (SVG)](docs/architecture-diagram.svg)
