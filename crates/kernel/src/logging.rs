use core::fmt;
use core::fmt::Write;

use spin::Mutex;
use uart_16550::SerialPort;

static SERIAL1: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(0x3F8) });

pub fn init() {
    SERIAL1.lock().init();
}

pub fn _print(args: fmt::Arguments<'_>) {
    let _ = SERIAL1.lock().write_fmt(args);
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::logging::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! kprintln {
    () => {
        $crate::kprint!("\n");
    };
    ($fmt:expr) => {
        $crate::kprint!(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::kprint!(concat!($fmt, "\n"), $($arg)*);
    };
}
