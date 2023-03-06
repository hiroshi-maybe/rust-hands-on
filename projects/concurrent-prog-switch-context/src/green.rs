use nix::sys::mman::{mprotect, ProtFlags};
use nix::unistd::SysconfVar;
use std::alloc::{alloc, Layout};
use std::ffi::c_void;

fn get_page_size() -> usize {
    // 4KiB in my Mac, that is the same value as Linux
    nix::unistd::sysconf(SysconfVar::PAGE_SIZE)
        .unwrap()
        .unwrap() as usize
}

#[derive(Debug)]
#[repr(C)]
pub struct Registers {
    // should be preserved for calling function - start
    rbx: u64,
    rbp: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    // should be preserved for calling function - end
    rsp: u64,
    rdx: u64,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rsp: 0,
            rdx: 0,
        }
    }

    pub fn new_with_stack(stack_size: usize, rip: u64) -> Self {
        let layout = Layout::from_size_align(stack_size, get_page_size()).unwrap();
        let stack = unsafe { alloc(layout) };
        println!(
            "start: {}, end: {}, page size: {}, layout: {:?}",
            stack as u64,
            stack as u64 + stack_size as u64,
            get_page_size(),
            layout
        );

        // Protect stack for potential stack overflow
        unsafe { mprotect(stack as *mut c_void, get_page_size(), ProtFlags::PROT_NONE).unwrap() };

        Registers {
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rsp: stack as u64 + stack_size as u64,
            rdx: rip,
        }
    }
}
