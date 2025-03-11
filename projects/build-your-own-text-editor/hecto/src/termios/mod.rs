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
const ISIG: c_ulong = 0x00000080;
const IXON: c_ulong = 0x00000200;
const IEXTEN: c_ulong = 0x00000400;
const ICRNL: c_ulong = 0x00000100;
const OPOST: c_ulong = 0x00000001;
const BRKINT: c_ulong = 0x00000002;
const INPCK: c_ulong = 0x00000010;
const ISTRIP: c_ulong = 0x00000020;
const CS8: c_ulong = 0x00000300;
const LFLAG_MASK: c_ulong =
    ECHO | ICANON | ISIG | IXON | IEXTEN | ICRNL | OPOST | BRKINT | INPCK | ISTRIP | CS8;

const STDIN_FILENO: c_int = 0;
const TCSAFLUSH: c_int = 2;

const VMIN: usize = 16;
const VTIME: usize = 17;

pub fn enable_raw_mode() -> Result<(), c_int> {
    update_termios_lflag(
        |lflag| lflag & !LFLAG_MASK,
        |mut cc| {
            cc[VMIN] = 0;
            cc[VTIME] = 1;
            cc
        },
    )?;
    register_exit_cleanup()?;

    Ok(())
}

pub fn disable_raw_mode() -> Result<(), c_int> {
    update_termios_lflag(|lflag| lflag | LFLAG_MASK, |cc| cc)
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

fn update_termios_lflag<T: FnOnce(c_ulong) -> c_ulong, U: FnOnce([u8; 20]) -> [u8; 20]>(
    f: T,
    g: U,
) -> Result<(), c_int> {
    unsafe {
        let mut termios = MaybeUninit::<Termios>::uninit();
        let result = tcgetattr(STDIN_FILENO, termios.as_mut_ptr());
        if result == -1 {
            return Err(result);
        }
        let mut termios = termios.assume_init();

        termios.c_lflag = f(termios.c_lflag);
        termios.c_cc = g(termios.c_cc);

        let result = tcsetattr(STDIN_FILENO, TCSAFLUSH, &termios);
        if result == -1 {
            return Err(result);
        }

        Ok(())
    }
}
