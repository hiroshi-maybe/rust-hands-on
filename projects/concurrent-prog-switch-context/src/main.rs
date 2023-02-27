use green::Registers;
use volatile::Volatile;

/// References
/// * https://graphitemaster.github.io/fibers/
mod green;

extern "C" {
    fn get_context(ctx: *mut Registers) -> u64;
    fn set_context(ctx: *const Registers) -> !;
}

fn main() {
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
