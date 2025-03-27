use std::{
    fs::File,
    os::{fd::AsRawFd, raw::c_int},
};

extern "C" {
    pub fn ftruncate(fd: c_int, length: i64) -> c_int;
}

pub fn truncate_file(cmd: &mut File, length: usize) -> Result<(), std::io::Error> {
    let res = unsafe { ftruncate(cmd.as_raw_fd(), length as i64) };

    if res != 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
