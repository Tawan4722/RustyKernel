use core::arch::asm;

use x86_64::registers::rflags::RFlags;
use xmas_elf::ElfFile;
use xmas_elf::program::Type;

use crate::boot::BootSnapshot;
use crate::gdt;
use crate::memory;
use crate::syscall;

const USER_STACK_TOP: u64 = 0x0000_7fff_ff00_0000;
const USER_STACK_PAGES: usize = 8;
const ENTER_RING3: bool = true;

pub fn launch_user_smoke(snapshot: &BootSnapshot) -> Result<(), &'static str> {
    let module = snapshot
        .modules
        .iter()
        .copied()
        .find(|m| {
            m.path()
                .to_str()
                .ok()
                .map(|v| v.contains("user-smoke"))
                .unwrap_or(false)
        })
        .ok_or("user-smoke module not found")?;

    let bytes =
        unsafe { core::slice::from_raw_parts(module.addr().cast::<u8>(), module.size() as usize) };
    let elf = ElfFile::new(bytes).map_err(|_| "invalid user-smoke ELF")?;
    let entry = elf.header.pt2.entry_point();

    for program_header in elf.program_iter() {
        let p_type = program_header
            .get_type()
            .map_err(|_| "bad program header")?;
        if p_type != Type::Load {
            continue;
        }

        let virt_start = program_header.virtual_addr();
        let mem_size = program_header.mem_size();
        let file_size = program_header.file_size();
        let offset = program_header.offset() as usize;

        if mem_size == 0 {
            continue;
        }

        map_user_range(virt_start, mem_size)?;
        let file_end = offset + file_size as usize;
        if file_end > bytes.len() {
            return Err("ELF segment out of bounds");
        }
        unsafe {
            memory::write_to_virtual(virt_start, &bytes[offset..file_end]);
        }
        if mem_size > file_size {
            zero_user_range(virt_start + file_size, mem_size - file_size);
        }
    }

    let user_stack_base = USER_STACK_TOP - (USER_STACK_PAGES as u64 * 4096);
    map_user_range(user_stack_base, USER_STACK_PAGES as u64 * 4096)?;

    crate::kprintln!(
        "[user] module={} bytes={} entry=0x{:x}",
        module.path().to_str().unwrap_or("<invalid-path>"),
        bytes.len(),
        entry
    );

    if ENTER_RING3 {
        unsafe {
            enter_ring3(entry, USER_STACK_TOP - 16);
        }
    }

    if !syscall::user_exited() {
        return Err("user-smoke did not signal exit");
    }

    Ok(())
}

fn map_user_range(start: u64, size: u64) -> Result<(), &'static str> {
    let end = start + size;
    let mut addr = start & !0xfff;
    let end_page = (end + 4095) & !0xfff;
    while addr < end_page {
        unsafe {
            memory::map_user_page(addr, true).map_err(|_| "user map failed")?;
        }
        addr += 4096;
    }
    Ok(())
}

fn zero_user_range(start: u64, size: u64) {
    let mut remaining = size;
    let mut cursor = start;
    static ZERO_CHUNK: [u8; 256] = [0; 256];
    while remaining > 0 {
        let n = core::cmp::min(remaining as usize, ZERO_CHUNK.len());
        unsafe {
            memory::write_to_virtual(cursor, &ZERO_CHUNK[..n]);
        }
        cursor += n as u64;
        remaining -= n as u64;
    }
}

unsafe fn enter_ring3(entry: u64, user_stack: u64) -> ! {
    let (user_cs, user_ss) = gdt::user_segment_values();
    let rflags = (RFlags::INTERRUPT_FLAG).bits();

    asm!(
        "push {user_ss}",
        "push {user_stack}",
        "push {rflags}",
        "push {user_cs}",
        "push {entry}",
        "iretq",
        user_ss = in(reg) user_ss,
        user_stack = in(reg) user_stack,
        rflags = in(reg) rflags,
        user_cs = in(reg) user_cs,
        entry = in(reg) entry,
        options(noreturn)
    )
}
