# Boot Flow

1. Limine loads `kernel.elf` and `user-smoke.elf`.
2. Kernel `_start` initializes serial logging and validates Limine request responses.
3. Kernel initializes:
   - HHDM-backed paging access
   - GDT/TSS and IDT
   - PIC and timer interrupts
   - Heap allocator
   - Scheduler and syscall handlers
4. Kernel loads the user smoke ELF into user-mapped pages, maps a user stack, and performs ring3 transition.
5. User smoke issues `int 0x80` twice (`write`, `exit` smoke sequence).
