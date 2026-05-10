# Boot Contract

Required files in EFI payload root:

- `limine.conf`
- `kernel.elf`
- `user-smoke.elf`

Required EFI path:

- `EFI/BOOT/BOOTX64.EFI` (Limine UEFI binary)

Limine config contract:

- Protocol: `limine`
- Kernel path: `boot():/kernel.elf`
- Module path: `boot():/user-smoke.elf`

Runtime assumptions:

- x86_64 CPU
- UEFI firmware (QEMU+OVMF or physical UEFI machine)
