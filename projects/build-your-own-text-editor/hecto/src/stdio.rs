use std::os::raw::c_int;
use std::os::unix::io::RawFd;

extern "C" {
    fn write(fd: c_int, buf: *const u8, count: usize) -> isize;
}

const STDOUT_FILENO: RawFd = 1;

pub fn write_command(cmd: &[u8]) -> Result<(), std::io::Error> {
    let res = unsafe { write(STDOUT_FILENO, cmd.as_ptr(), cmd.len()) };

    if res != cmd.len() as isize {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
