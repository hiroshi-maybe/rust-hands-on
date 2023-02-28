use std::process::exit;

use green::Registers;
use volatile::Volatile;

/// References
/// * https://graphitemaster.github.io/fibers/
mod green;

extern "C" {
    fn get_context(ctx: *mut Registers) -> u64;
    fn set_context(ctx: *const Registers);
    fn switch_context(ctx1: *const Registers, ctx2: *mut Registers);
}

#[allow(dead_code)]
fn repeat_context() {
    let mut x = 0;
    let mut volatile = Volatile::new(&mut x);

    let mut regs = Registers::new();
    let r = &mut regs as *mut Registers;
    unsafe {
        get_context(r);
    }
    println!("{:?}", regs);
    println!("hello, context switch!");

    if volatile.read() == 0 {
        volatile.write(1);
        unsafe {
            set_context(r);
        }
    }
}

fn foo() {
    println!("foo called");

    // No return address. So SEGV occurs without this
    exit(1);
}

#[allow(dead_code)]
fn switch_to_foo() {
    let mut regs = Registers::new_with_stack(2 * 1024 * 1024, foo as u64);
    println!("{:?}", regs);
    let r = &mut regs as *mut Registers;

    unsafe {
        set_context(r);
    }
}

static mut REG_MAIN: Option<Box<Registers>> = None;
fn foo2() {
    println!("foo called");

    unsafe {
        if let Some(r1) = &mut REG_MAIN {
            set_context(&mut **r1 as *mut Registers);
        }
    }
}
#[allow(dead_code)]
fn switch_to_foo2() {
    let regs1 = Registers::new();
    let mut regs2 = Registers::new_with_stack(2 * 1024 * 1024, foo2 as u64);

    unsafe {
        REG_MAIN = Some(Box::new(regs1));
        if let Some(r1) = &mut REG_MAIN {
            switch_context(&mut regs2 as *mut Registers, &mut **r1 as *mut Registers)
        }
    }
}

fn main() {
    // repeat_context();
    // switch_to_foo();
    switch_to_foo2();
    println!("main ended");
}
