#![no_std]
#![no_main]

extern crate alloc;

use core::alloc::Layout;
use core::panic::PanicInfo;

mod boot;
mod gdt;
mod interrupts;
mod logging;
mod memory;
mod scheduler;
mod syscall;
mod userspace;

use x86_64::instructions::hlt;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    logging::init();
    kprintln!("[kernel] entering _start");

    let snapshot = match boot::snapshot() {
        Ok(v) => v,
        Err(err) => panic!("boot snapshot failed: {err}"),
    };

    memory::init_boot_memory(snapshot.hhdm_offset, snapshot.memory_map);
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::initialize_pic() };
    x86_64::instructions::interrupts::enable();

    memory::init_heap();
    scheduler::init();
    syscall::init();

    scheduler::register_kernel_task("idle-log", scheduler::idle_log_task);
    scheduler::register_kernel_task("stats", scheduler::stats_task);
    scheduler::register_kernel_task("user-exit", syscall::process_control_task);

    let user_result = userspace::launch_user_smoke(&snapshot);
    match user_result {
        Ok(()) => kprintln!("[kernel] user smoke task exited cleanly"),
        Err(err) => kprintln!("[kernel] user smoke launch failed: {}", err),
    }

    kprintln!("[kernel] boot complete");
    hlt_loop()
}

fn hlt_loop() -> ! {
    loop {
        scheduler::run_tick();
        hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    kprintln!("[panic] {}", info);
    hlt_loop()
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation failure: {:?}", layout);
}
