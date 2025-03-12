use std::os::raw::c_int;
use std::os::unix::io::RawFd;

extern "C" {
    fn write(fd: c_int, buf: *const u8, count: usize) -> isize;
}

const STDOUT_FILENO: RawFd = 1;

pub fn refresh_screen() {
    let clear_screen_cmd = b"\x1b[2J";
    write_command(clear_screen_cmd);
    let reposition_cursor_cmd = b"\x1b[H";
    write_command(reposition_cursor_cmd);
}

fn write_command(cmd: &[u8]) {
    unsafe {
        write(STDOUT_FILENO, cmd.as_ptr(), cmd.len());
    }
}
