/*

  Five actions in unsafe Rust, called unsafe superpowers, that you can’t in safe Rust. Those superpowers include the ability to:

    - Dereference a raw pointer
    - Call an unsafe function or method
    - Access or modify a mutable static variable
    - Implement an unsafe trait
    - Access fields of unions

 */

fn main() {
    {
        // Dereferencing a Raw Pointer

        let mut num = 5;

        let r1 = &num as *const i32; // immutable raw pointer
        let r2 = &mut num as *mut i32; // mutable raw pointer

        // raw pointer from a memory address
        let address = 0x012345usize;
        let _ = address as *const i32;

        unsafe {
            println!("r1 is: {} at {:?}", *r1, r1);
            println!("r2 is: {} at {:?}", *r2, r2);
        }
        // `dereference of raw pointer is unsafe and requires unsafe function or block`
        // println!("r1 is: {}", *r1);
    }

    {
        // Calling an Unsafe Function or Method

        unsafe fn dangerous() {}
        unsafe {
            dangerous();
        }

        fn split_at_mut(slice: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
            /*
            // `cannot borrow `*slice` as mutable more than once at a time`
            let len = slice.len();
            assert!(mid <= len);
            (&mut slice[..mid], &mut slice[mid..])
            */
            let len = slice.len();
            let ptr = slice.as_mut_ptr();

            assert!(mid <= len);

            unsafe {
                use std::slice;
                (
                    // `from_raw_parts_mut()` is an unsafe function
                    slice::from_raw_parts_mut(ptr, mid),
                    slice::from_raw_parts_mut(ptr.add(mid), len - mid),
                )
            }
        }

        let mut v = vec![1,2,3,4,5,6];
        let r = &mut v[..];
        let (a, b) = split_at_mut(r, 3);
        assert_eq!(a, &mut [1,2,3]);
        assert_eq!(b, &mut [4,5,6]);
    }

    {
        // Using extern Functions to Call External Code

        extern "C" {
            fn abs(input: i32) -> i32;
        }

        unsafe {
            println!("Absolute value of -3 according to C: {}", abs(-3));
        }

        #[no_mangle]
        pub extern "C" fn call_from_c() {
            println!("Just called a Rust function from C!");
        }
    }

    {
        // Accessing or Modifying a Mutable Static Variable

        println!("name is: {}", HELLO_WORLD);

        fn add_to_count(inc: u32) {
            unsafe {
                COUNTER += inc;
            }
        }

        add_to_count(3);
        unsafe {
            println!("COUNTER: {}", COUNTER);
        }

        // `use of mutable static is unsafe and requires unsafe function or block`
        //println!("COUNTER: {}", COUNTER);
    }

    {
        // Implementing an Unsafe Trait

        unsafe trait Foo {
            // methods go here
        }

        unsafe impl Foo for i32 {
            // method implementations go here
        }
    }
}

// Accessing or Modifying a Mutable Static Variable
static HELLO_WORLD: &str = "Hello, world!";
static mut COUNTER: u32 = 0;