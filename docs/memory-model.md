# Memory Model

- Limine memory map is requested at boot.
- Usable regions are collected into a fixed-size frame allocator.
- Frame allocator hands out `4KiB` physical frames sequentially.
- Kernel heap:
  - Virtual base: `0xFFFF_9000_0000_0000`
  - Size: `1 MiB`
  - Backed by mapped writable pages
- User pages are mapped as `PRESENT | USER_ACCESSIBLE | WRITABLE` for smoke execution.

Known limitations:
- No frame deallocation in v1.
- No copy-on-write or per-process address spaces yet.
