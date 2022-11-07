use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
// use std::io::{self, Write};
use std::net::TcpListener;
use std::os::unix::prelude::{AsRawFd, RawFd};
// use std::os::unix::io::AsRawFd;
use std::ptr;

// Refered to https://nima101.github.io/kqueue_server
// $ socat stdio tcp:localhost:10000

fn main() {
    let listener = TcpListener::bind("127.0.0.1:10000").unwrap();
    let listener_fd = listener.as_raw_fd();

    let kq = unsafe { ffi::kqueue() };
    if kq < 0 {
        panic!("Cannot create a queue: {}", std::io::Error::last_os_error());
    }

    let ev = ffi::Kevent {
        ident: listener_fd as u64,
        filter: ffi::EVFILT_READ,
        flags: ffi::EV_ADD,
        fflags: 0,
        data: 0,
        udata: 1234,
    };
    let read_events = vec![ev];
    let res = unsafe { ffi::kevent(kq, read_events.as_ptr(), 1, ptr::null_mut(), 0, ptr::null()) };

    println!("kq: {}, listen_fd: {}", kq, listener_fd);

    if res < 0 {
        panic!(
            "Cannot register events: {}",
            std::io::Error::last_os_error()
        );
    }

    let mut fd2buf = HashMap::new();

    loop {
        let mut events: Vec<ffi::Kevent> = Vec::with_capacity(1000);

        let res = unsafe {
            ffi::kevent(
                kq,
                ptr::null(),
                0,
                events.as_mut_ptr(),
                events.capacity() as i32,
                ptr::null(),
            )
        };
        if res < 0 {
            break;
        }

        unsafe { events.set_len(res as usize) };

        for n in 0..res {
            let e = &events[n as usize];
            if e.ident == listener_fd as u64 {
                if let Ok((stream, _)) = listener.accept() {
                    let stream_fd = stream.as_raw_fd();
                    let s = stream.try_clone().unwrap();
                    let reader = BufReader::new(s);
                    let writer = BufWriter::new(stream);
                    fd2buf.insert(stream_fd, (reader, writer));

                    println!("fd {}: connection established", stream_fd);

                    let ev = ffi::Kevent {
                        ident: stream_fd as u64,
                        filter: ffi::EVFILT_READ,
                        flags: ffi::EV_ADD,
                        fflags: 0,
                        data: 0,
                        udata: 0,
                    };
                    let read_events = vec![ev];
                    let res = unsafe {
                        ffi::kevent(kq, read_events.as_ptr(), 1, ptr::null_mut(), 0, ptr::null())
                    };
                    if res < 0 {
                        panic!(
                            "Cannot register events upon accept ({}): {}",
                            res,
                            std::io::Error::last_os_error()
                        );
                    }
                }
            } else {
                let fd = events[n as usize].ident as RawFd;
                let (reader, writer) = fd2buf.get_mut(&fd).unwrap();

                let mut buf = String::new();
                let n = reader.read_line(&mut buf).unwrap();

                // connectin closed
                if n == 0 {
                    let ev = ffi::Kevent {
                        ident: fd as u64,
                        filter: ffi::EVFILT_READ,
                        flags: ffi::EV_DELETE,
                        fflags: 0,
                        data: 0,
                        udata: 0,
                    };
                    let read_events = vec![ev];
                    let res = unsafe {
                        ffi::kevent(kq, read_events.as_ptr(), 1, ptr::null_mut(), 0, ptr::null())
                    };
                    if res < 0 {
                        panic!("Cannot delete events: {}", std::io::Error::last_os_error());
                    }
                    println!("fd {}: connection closed", fd);
                    unsafe { ffi::close(fd) };
                    continue;
                }

                println!("fd {}: read data '{}'", fd, buf.trim_end());

                writer.write(buf.as_bytes()).unwrap();
                writer.flush().unwrap();
            }
        }
    }
}

// https://cfsamsonbooks.gitbook.io/epoll-kqueue-iocp-explained/part-1-an-express-explanation/kqueue-the-express-version

mod ffi {
    pub const EVFILT_READ: i16 = -1;
    pub const EV_ADD: u16 = 0x1;
    // pub const EV_ENABLE: u16 = 0x4;
    // pub const EV_ONESHOT: u16 = 0x10;
    pub const EV_DELETE: u16 = 0x2;

    #[derive(Debug)]
    #[repr(C)]
    pub(super) struct Timespec {
        /// Seconds
        tv_sec: isize,
        /// Nanoseconds
        v_nsec: usize,
    }

    impl Timespec {
        #[allow(dead_code)]
        pub fn from_millis(milliseconds: i32) -> Self {
            let seconds = milliseconds / 1000;
            let nanoseconds = (milliseconds % 1000) * 1000 * 1000;
            Timespec {
                tv_sec: seconds as isize,
                v_nsec: nanoseconds as usize,
            }
        }
    }

    // https://github.com/rust-lang/libc/blob/c8aa8ec72d631bc35099bcf5d634cf0a0b841be0/src/unix/bsd/apple/mod.rs#L497
    // https://github.com/rust-lang/libc/blob/c8aa8ec72d631bc35099bcf5d634cf0a0b841be0/src/unix/bsd/apple/mod.rs#L207
    #[derive(Debug, Clone, Default)]
    #[repr(C)]
    pub struct Kevent {
        pub ident: u64,
        pub filter: i16,
        pub flags: u16,
        pub fflags: u32,
        pub data: i64,
        pub udata: u64,
    }

    #[link(name = "c")]
    extern "C" {
        /// Returns: positive: file descriptor, negative: error
        pub(super) fn kqueue() -> i32;

        pub(super) fn kevent(
            kq: i32,
            changelist: *const Kevent,
            nchanges: i32,
            eventlist: *mut Kevent,
            nevents: i32,
            timeout: *const Timespec,
        ) -> i32;

        pub fn close(d: i32) -> i32;
    }
}
