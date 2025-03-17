use std::os::raw::c_int;
use std::os::unix::io::RawFd;

pub struct BufferedCommands {
    buffer: Vec<u8>,
}

impl BufferedCommands {
    pub fn new(commands: Vec<u8>) -> Self {
        Self { buffer: commands }
    }

    pub fn append(&mut self, cmd: &[u8]) {
        self.buffer.extend_from_slice(cmd);
    }

    pub fn execute(&mut self) -> Result<(), std::io::Error> {
        write_command(&self.buffer)?;
        self.buffer.clear();
        Ok(())
    }
}

extern "C" {
    fn write(fd: c_int, buf: *const u8, count: usize) -> isize;
}

const STDOUT_FILENO: RawFd = 1;

fn write_command(cmd: &[u8]) -> Result<(), std::io::Error> {
    let res = unsafe { write(STDOUT_FILENO, cmd.as_ptr(), cmd.len()) };

    if res != cmd.len() as isize {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
