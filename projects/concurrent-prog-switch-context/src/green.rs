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
}
