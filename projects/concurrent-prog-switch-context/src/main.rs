use std::process::exit;

use green::Registers;
use volatile::Volatile;

/// References
/// * https://graphitemaster.github.io/fibers/
/// * https://dev.to/bmatcuk/debugging-rust-with-rust-lldb-j1f
/// * https://stackoverflow.com/questions/71106599/understanding-16-byte-padding-and-function-prolog-in-x64-assembly
mod green;

extern "C" {
    fn get_context(ctx: *mut Registers) -> u64;
    fn jump_to_func(ctx: *const Registers);
    fn restore_old_context(ctx: *const Registers);
    fn switch_context(ctx1: *const Registers, ctx2: *mut Registers);
}

macro_rules! debug_reg {
    () => {
        let mut reg = Registers::new();
        let r = &mut reg as *mut Registers;
        unsafe {
            get_context(r);
        }
        println!("[DEBUG] curent reg: {:?}", reg);
        unsafe {
            assert_eq!((*r).rbp % 16, 0);
        }
    };
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
            restore_old_context(r);
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
        jump_to_func(r);
    }
}

pub fn hashmap_random_keys() -> (u64, u64) {
    const KEY_LEN: usize = core::mem::size_of::<u64>();
    println!("hashmap_random_keys 1: {}", KEY_LEN);
    debug_reg!();
    let mut v = [0u8; KEY_LEN * 2];
    // fill_bytes(&mut v);

    println!("hashmap_random_keys 2");
    let key1 = v[0..KEY_LEN].try_into().unwrap();
    let key2 = v[KEY_LEN..].try_into().unwrap();

    println!("hashmap_random_keys 3");

    (u64::from_ne_bytes(key1), u64::from_ne_bytes(key2))
}

// fn getentropy_fill_bytes(v: &mut [u8]) -> bool {
//     weak!(fn getentropy(*mut c_void, size_t) -> c_int);

//     getentropy
//         .get()
//         .map(|f| {
//             // getentropy(2) permits a maximum buffer size of 256 bytes
//             for s in v.chunks_mut(256) {
//                 let ret = unsafe { f(s.as_mut_ptr() as *mut c_void, s.len()) };
//                 if ret == -1 {
//                     panic!("unexpected getentropy error: {}", errno());
//                 }
//             }
//             true
//         })
//         .unwrap_or(false)
// }

// pub fn fill_bytes(v: &mut [u8]) {
//     if getentropy_fill_bytes(v) {
//         return;
//     }

//     // for older macos which doesn't support getentropy
//     let mut file = File::open("/dev/urandom").expect("failed to open /dev/urandom");
//     file.read_exact(v).expect("failed to read /dev/urandom")
// }

static mut REG_MAIN: Option<Box<Registers>> = None;
fn bar() {
    println!("bar called");
    debug_reg!();

    const KEY_LEN: usize = core::mem::size_of::<u64>();
    let x = [0u8; 16];
    println!("{}, {:?}", KEY_LEN, x);

    let x = hashmap_random_keys();
    println!("{:?}", x);

    // let x = std::sys::rand::hashmap_random_keys();

    // SEGV
    // let _: HashSet<usize> = HashSet::new();

    // No SEGV
    // let xs = vec![1, 2];
    // println!("{:?}", xs);

    // set.insert(1);
    // let x = set.contains(&1);
    // println!("contains called: {}", x);

    unsafe {
        if let Some(r1) = &mut REG_MAIN {
            println!("switch to: {:?}", *r1);
            restore_old_context(&mut **r1 as *mut Registers);
        }
    }
}
#[allow(dead_code)]
fn switch_to_foo2() {
    let regs1 = Registers::new();
    let mut regs2 = Registers::new_with_stack(32 * 1024 * 1024, bar as u64);

    const KEY_LEN: usize = core::mem::size_of::<u64>();
    let x = [0u8; KEY_LEN * 2];
    println!("{:?}", x);

    unsafe {
        REG_MAIN = Some(Box::new(regs1));
        if let Some(r1) = &mut REG_MAIN {
            // let set: HashSet<usize> = HashSet::new();
            // println!("{:?}", set);
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
