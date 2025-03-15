use std::os::{
    fd::RawFd,
    raw::{c_int, c_ulong, c_ushort},
};

#[repr(C)]
#[derive(Debug)]
struct Winsize {
    ws_row: c_ushort,
    ws_col: c_ushort,
    ws_xpixel: c_ushort,
    ws_ypixel: c_ushort,
}

const STDOUT_FILENO: RawFd = 1;
const TIOCGWINSZ: c_ulong = 0x40087468;

extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
}

pub fn get_window_size() -> Result<(usize, usize), std::io::Error> {
    unsafe {
        let mut winsize = std::mem::MaybeUninit::<Winsize>::uninit();

        if ioctl(STDOUT_FILENO, TIOCGWINSZ, winsize.as_mut_ptr()) == -1 {
            return Err(std::io::Error::last_os_error());
        }

        let winsize = winsize.assume_init();
        dbg!(winsize.ws_row, winsize.ws_col);
        Ok((winsize.ws_row as usize, winsize.ws_col as usize))
    }
}
