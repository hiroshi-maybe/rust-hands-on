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
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    // should be preserved for calling function - end
    pub rsp: u64,
    pub rdx: u64,
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

        let stack_bottom = stack as u64 + stack_size as u64;
        println!(
            "start: {}, end: {}, page size: {}, layout: {:?}",
            stack as u64,
            stack_bottom,
            get_page_size(),
            layout
        );
        assert_eq!(stack_bottom % 16, 0);

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
