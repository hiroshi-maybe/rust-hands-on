use std::mem::MaybeUninit;
use std::os::raw::{c_int, c_ulong};
use std::sync::Mutex;

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
const IFLAG_MASK: c_ulong = BRKINT | ICRNL | INPCK | ISTRIP | IXON;
const OFLAG_MASK: c_ulong = OPOST;
const CFLAG_MASK: c_ulong = CS8;
const LFLAG_MASK: c_ulong = ECHO | ICANON | IEXTEN | ISIG;

const STDIN_FILENO: c_int = 0;
const TCSAFLUSH: c_int = 2;

const VMIN: usize = 16;
const VTIME: usize = 17;

pub fn enable_raw_mode() -> Result<(), c_int> {
    let mut original_termios = TERMINAL_MODE_PRIOR_RAW_MODE.lock().unwrap();
    if original_termios.is_some() {
        println!("raw mode already enabled");
        return Ok(());
    }

    *original_termios = get_termios().ok();

    update_termios(
        |iflag| iflag & !IFLAG_MASK,
        |oflag| oflag & !OFLAG_MASK,
        |cflag| cflag | CFLAG_MASK,
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

fn disable_raw_mode() -> Result<(), c_int> {
    let mut original_mode = TERMINAL_MODE_PRIOR_RAW_MODE.lock().unwrap();
    if let Some(termios) = original_mode.take() {
        set_termios(&termios)?;
        *original_mode = None;
    }

    Ok(())
}

// Some(Termios) -> we're in the raw mode and this is the previous mode
// None -> we're not in the raw mode
static TERMINAL_MODE_PRIOR_RAW_MODE: Mutex<Option<Termios>> = Mutex::new(None);

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

fn update_termios(
    i_flag: impl FnOnce(c_ulong) -> c_ulong,
    o_flag: impl FnOnce(c_ulong) -> c_ulong,
    c_flag: impl FnOnce(c_ulong) -> c_ulong,
    l_flag: impl FnOnce(c_ulong) -> c_ulong,
    cc: impl FnOnce([u8; 20]) -> [u8; 20],
) -> Result<(), c_int> {
    let mut termios = get_termios()?;

    termios.c_iflag = i_flag(termios.c_iflag);
    termios.c_oflag = o_flag(termios.c_oflag);
    termios.c_cflag = c_flag(termios.c_cflag);
    termios.c_lflag = l_flag(termios.c_lflag);
    termios.c_cc = cc(termios.c_cc);

    set_termios(&termios)
}

fn get_termios() -> Result<Termios, c_int> {
    unsafe {
        let mut termios = MaybeUninit::<Termios>::uninit();
        let result = tcgetattr(STDIN_FILENO, termios.as_mut_ptr());
        if result == -1 {
            return Err(result);
        }

        Ok(termios.assume_init())
    }
}

fn set_termios(termios: &Termios) -> Result<(), c_int> {
    unsafe {
        let result = tcsetattr(STDIN_FILENO, TCSAFLUSH, termios);
        if result == -1 {
            return Err(result);
        }

        Ok(())
    }
}
