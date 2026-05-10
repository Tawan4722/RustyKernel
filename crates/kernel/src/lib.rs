#![no_std]

pub mod abi {
    pub const SYSCALL_WRITE: usize = 1;
    pub const SYSCALL_EXIT: usize = 2;
    pub const SYSCALL_YIELD: usize = 3;

    pub const ERRNO_INVAL: isize = -22;
    pub const ERRNO_NOSYS: isize = -38;
}
