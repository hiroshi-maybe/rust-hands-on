use std::mem::MaybeUninit;
use std::os::raw::{c_int, c_ulong};

#[repr(C)]
#[derive(Clone, Copy)]
struct Termios {
    c_iflag: c_ulong,
    c_oflag: c_ulong,
    c_cflag: c_ulong,
    c_lflag: c_ulong,
    c_cc: [u8; 20],
    c_ispeed: c_ulong,
    c_ospeed: c_ulong,
}

extern "C" {
    fn tcgetattr(fd: c_int, termios_p: *mut Termios) -> c_int;
    fn tcsetattr(fd: c_int, optional_actions: c_int, termios_p: *const Termios) -> c_int;
}

const STDIN_FILENO: c_int = 0;
const TCSANOW: c_int = 0;

pub fn enable_raw_mode() -> Result<(), c_int> {
    unsafe {
        let mut termios = MaybeUninit::<Termios>::uninit();
        let result = tcgetattr(STDIN_FILENO, termios.as_mut_ptr());
        if result == -1 {
            return Err(result);
        }
        let mut termios = termios.assume_init();

        dbg!(result);
        println!("Before modification: c_lflag = {:X}", termios.c_lflag);

        termios.c_lflag &= !0o10;

        dbg!(termios.c_lflag);

        let result = tcsetattr(STDIN_FILENO, TCSANOW, &termios);
        if result == -1 {
            return Err(result);
        }

        println!("After modification: c_lflag = {:X}", termios.c_lflag);

        Ok(())
    }
}
