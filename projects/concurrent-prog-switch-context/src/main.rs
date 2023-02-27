use std::process::exit;

use green::Registers;
use volatile::Volatile;

/// References
/// * https://graphitemaster.github.io/fibers/
mod green;

extern "C" {
    fn get_context(ctx: *mut Registers) -> u64;
    fn set_context(ctx: *const Registers) -> !;
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
    let r = &mut regs as *mut Registers;

    unsafe {
        set_context(r);
    }
}

fn main() {
    // repeat_context();
    switch_to_foo();
}
