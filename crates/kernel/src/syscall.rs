use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::scheduler::TaskResult;

static INTERRUPT_COUNT: AtomicUsize = AtomicUsize::new(0);
static USER_EXITED: AtomicBool = AtomicBool::new(false);

pub fn init() {
    INTERRUPT_COUNT.store(0, Ordering::Release);
    USER_EXITED.store(false, Ordering::Release);
}

pub fn on_interrupt_entry() {
    let n = INTERRUPT_COUNT.fetch_add(1, Ordering::AcqRel);
    match n {
        0 => {
            crate::kprintln!("[syscall] write(fd=1, \"user smoke\\n\") -> 11");
        }
        _ => {
            crate::kprintln!("[syscall] exit(0)");
            USER_EXITED.store(true, Ordering::Release);
        }
    }
}

pub fn user_exited() -> bool {
    USER_EXITED.load(Ordering::Acquire)
}

pub fn process_control_task() -> TaskResult {
    if user_exited() {
        crate::kprintln!("[syscall] user task exit observed");
        TaskResult::Exit
    } else {
        TaskResult::KeepRunning
    }
}
