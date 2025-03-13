use std::os::raw::c_int;
use std::os::unix::io::RawFd;

extern "C" {
    fn write(fd: c_int, buf: *const u8, count: usize) -> isize;
}

const STDOUT_FILENO: RawFd = 1;

pub fn write_command(cmd: &[u8]) {
    unsafe {
        write(STDOUT_FILENO, cmd.as_ptr(), cmd.len());
    }
}
