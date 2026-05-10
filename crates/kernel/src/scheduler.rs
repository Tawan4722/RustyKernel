use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use spin::Mutex;

use crate::memory;

const MAX_TASKS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskResult {
    KeepRunning,
    Exit,
}

type TaskFn = fn() -> TaskResult;

#[derive(Clone, Copy)]
struct Task {
    name: &'static str,
    entry: TaskFn,
    active: bool,
}

impl Task {
    const fn empty() -> Self {
        Self {
            name: "",
            entry: idle_log_task,
            active: false,
        }
    }
}

struct Scheduler {
    tasks: [Task; MAX_TASKS],
    next_index: usize,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            tasks: [Task::empty(); MAX_TASKS],
            next_index: 0,
        }
    }

    fn add_task(&mut self, name: &'static str, entry: TaskFn) {
        for slot in &mut self.tasks {
            if !slot.active {
                *slot = Task {
                    name,
                    entry,
                    active: true,
                };
                return;
            }
        }
    }

    fn run_one(&mut self) {
        for _ in 0..MAX_TASKS {
            let idx = self.next_index;
            self.next_index = (self.next_index + 1) % MAX_TASKS;
            if !self.tasks[idx].active {
                continue;
            }

            let result = (self.tasks[idx].entry)();
            if result == TaskResult::Exit {
                self.tasks[idx].active = false;
            }
            return;
        }
    }
}

static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
static TICKS: AtomicU64 = AtomicU64::new(0);
static NEED_SCHEDULE: AtomicBool = AtomicBool::new(false);

pub fn init() {
    TICKS.store(0, Ordering::Release);
    NEED_SCHEDULE.store(true, Ordering::Release);
}

pub fn register_kernel_task(name: &'static str, entry: TaskFn) {
    SCHEDULER.lock().add_task(name, entry);
}

pub fn on_timer_tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
    NEED_SCHEDULE.store(true, Ordering::Release);
}

pub fn run_tick() {
    if NEED_SCHEDULE.swap(false, Ordering::AcqRel) {
        SCHEDULER.lock().run_one();
    }
}

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Acquire)
}

pub fn idle_log_task() -> TaskResult {
    if ticks() % 500 == 0 {
        crate::kprintln!("[sched] tick {}", ticks());
    }
    TaskResult::KeepRunning
}

pub fn stats_task() -> TaskResult {
    if ticks() % 1000 == 0 {
        crate::kprintln!(
            "[mem] allocated_frames={} ticks={}",
            memory::frame_allocation_count(),
            ticks()
        );
    }
    TaskResult::KeepRunning
}
