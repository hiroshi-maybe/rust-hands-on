use std::mem::MaybeUninit;
use std::os::raw::{c_int, c_ulong};

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg(target_os = "macos")]
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
    fn atexit(func: extern "C" fn()) -> c_int;
    fn tcgetattr(fd: c_int, termios_p: *mut Termios) -> c_int;
    fn tcsetattr(fd: c_int, optional_actions: c_int, termios_p: *const Termios) -> c_int;
}

const ECHO: c_ulong = 0o10;
const ICANON: c_ulong = 0x00000100;

const STDIN_FILENO: c_int = 0;
const TCSAFLUSH: c_int = 2;

pub fn enable_raw_mode() -> Result<(), c_int> {
    update_termios_lflag(|lflag| lflag & !(ECHO | ICANON))?;
    register_exit_cleanup()?;

    Ok(())
}

pub fn disable_raw_mode() -> Result<(), c_int> {
    update_termios_lflag(|lflag| lflag | ECHO)
}

extern "C" fn disable_raw_mode_on_exit() {
    disable_raw_mode().expect("failed to disable raw mode");
    println!("disabled raw mode on exit");
}

fn register_exit_cleanup() -> Result<(), c_int> {
    unsafe {
        let res = atexit(disable_raw_mode_on_exit);
        if res != 0 {
            return Err(res);
        }
    }

    Ok(())
}

fn update_termios_lflag<T: FnOnce(c_ulong) -> c_ulong>(f: T) -> Result<(), c_int> {
    unsafe {
        let mut termios = MaybeUninit::<Termios>::uninit();
        let result = tcgetattr(STDIN_FILENO, termios.as_mut_ptr());
        if result == -1 {
            return Err(result);
        }
        let mut termios = termios.assume_init();

        termios.c_lflag = f(termios.c_lflag);

        let result = tcsetattr(STDIN_FILENO, TCSAFLUSH, &termios);
        if result == -1 {
            return Err(result);
        }

        Ok(())
    }
}
