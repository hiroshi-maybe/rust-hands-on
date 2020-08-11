/*

  Data in stack is always copied (no ownership)
  There is ownership in data in heap

  1. Without reference

   1) Ownership is moved for assignment or function call
   2) The equal data can be passed without move by clone()

  2. With reference

   1) Data can be borrowed by `&var` for assignment or function call
   2) Borrowed data can be mutated by `&mut var` for assignment or function call

   - Multiple reference is allowed for only immutable borrows
   - Mutable reference cannot be mixed with other mutable or immutable borrows to prevent unexpected disruption (single mutable ownership).
   - However it's possible only when life of the others has ended after the last usage of them

*/


fn main() {
    {
        // Immutable string literal
        let s = "hello";
        // Compile error
        // s.push_str(", world!");

        println!("{}", s);
    }

    {
        // Mutable string allocated on the heap
        let mut s = String::from("hello");
        s.push_str(", world!");
        println!("{}", s);
    }

    {
        // Integer assignment
        let mut x = 5;
        let y = x; // integer is copied
        println!("x={}, y={}", x, y);

        x = 100;
        println!("x={}, y={}", x, y);
    }

    {
        // String assignment
        let s1 = String::from("hello");
        // `s1` is moved to `s2`. `s1` is invalidated.
        // only `s2` will be freed when it goes out of the scope.
        let s2 = s1;
        // Reference to `s1` throws an error because it's already invalidated.
        //println!("s1={}, s2={}", s1, s2);
        println!("s2={}", s2);

        let mut s3 = String::from("hello");
        // `s3` is moved to `s4`.
        let s4 = s3;
        // Mutable or immutable does not matter. `s3` is already invalidated.
        // s3.push_str(", world!");
        println!("s4={}", s4);
    }

    {
        // Clone works in above example.
        let s1 = String::from("hello");
        let s2 = s1.clone();
        println!("s1={}, s2={}", s1, s2);

        let mut s3 = String::from("hello");
        let s4 = s3.clone();
        s3.push_str(", world!");
        println!("s3={}, s4={}", s3, s4);

        // String literal has a fixed size. Thus it's stored in stack.
        // Since they are copied, compiler never throws an error.
        let s5 = "hello";
        let s6 = s5;
        println!("s5={}, copied s6={}", s5, s6);
    }

    {
        // Ownership and Functions

        let s = String::from("hello");
        println!("before moving to a function: {}", s);
        takes_ownership(s);
        // Ownership is moved to the `takes_ownership()`.
        // Compile-time error!
        // println!("{}", s);

        fn takes_ownership(s: String) {
            println!("moved to a function: {}", s);
        }

        let n = 5;
        makes_copy(n);
        // No Compile error because `n` is copied to `makes_copy()`
        println!("Copied value is still valid: {}", n);

        fn makes_copy(n: i32) {
            println!("Copied to a function: {}", n);
        }
    }

    {
        // Return Values and Scope

        let s1 = gives_ownership();

        fn gives_ownership() -> String {
            String::from("hello")
        }

        println!("given string along with ownership: {}", s1);

        let s2 = takes_and_gives_back(s1);
        // Compile-time error because s1 is moved to the function
        // println!("{}", s1);
        println!("given string along with ownership: {}", s2);

        fn takes_and_gives_back(s: String) -> String { s }
    }

    {
        // Reuse data by returning from a function with ownership
        let s1 = String::from("hello");
        let (s2, len) = calc_length(s1);
        println!("{} is returned back from a function with length {}", s2, len);

        fn calc_length(s: String) -> (String, usize) {
            // This gives compile-time error because s is moved in the first item of the tuple
            //(s, s.len())

            let l = s.len();
            (s, l)
        }

        let (len, s1) = calc_length2(s2);
        println!("{} is returned back from a function with length {}", s1, len);

        fn calc_length2(s: String) -> (usize, String) { (s.len(), s) }
    }

    {
        // Borrow data by reference
        let s1 = String::from("hello");
        let len = calc_length(&s1);
        println!("{} is borrowed by a function which returned length {}", s1, len);

        fn calc_length(s: &String) -> usize {
            // Cannot be mutated beause it's borrowed by immutable reference
            //s.push_str("world");

            // The function borrows the ownership by the reference
            s.len()
        }

        let s1 = "hello";
        let len = calc_length2(s1);
        println!("String literal {} forces borrowing it and returned length is {}", s1, len);

        fn calc_length2(s: &str) -> usize {
            s.len()
        }
    }

    {
        // Mutable references

        let mut s = String::from("hello");
        change(&mut s);
        change(&mut s);

        fn change(s: &mut String) {
            s.push_str(", world!");
        }

        println!("String was mutated through mutable reference: {}", s);
    }

    {
        let mut s = String::from("hello");

        {
            // Borrow can happen even in assignments
            let r1 = &mut s;
            // Cannot borrow twice
            //let r2 = &mut s;

            println!("{}", r1);
        }

        {
            {
                // Immutable -> mutable reference

                let r1 = &s;
                // Can be borrowed twice if it's immutable borrow
                let r2 = &s;
                // But mutable borrow is not allowed upon immutable borrow(s)
                // because mutably borrowing data may change it
                // while life of the immutable is still on-going
                //let r3 = &mut s;

                println!("{} and {}", r1, r2);

                // However this is safe because life of `r1` and `r2` has ended
                // at the last usage of them (println!() above in this case)
                let r3 = &mut s;
                r3.push_str(", world-1!");
                println!("{}", r3);
            }

            {
                // Mutable -> immutable reference
                let r1 = &mut s;

                r1.push_str(", world-2!");

                // Immutable borrow is allowed after the last usage of mutable `r1`.
                let r2 = &r1;
                let r3 = &r1;

                println!("{} and {}", r2, r3);
            }
        }
    }

    {
        // Dangling References

        /*
        let reference_to_nothing = dangle();
        // This function throws compile-time error because `s` is dropped outside of the function
        fn dangle() -> &String {
            let s = String::from("Hello");
            &s
        }*/

        let moved_string = no_dangle();

        fn no_dangle() -> String {
            let s = String::from("hello");
            s
        }
        println!("moved string from a function: {}", moved_string);

        let s = String::from("hello");
        let ss = borrowback(&s);
        println!("Original: {}, borrowed back: {}", s, ss);
        fn borrowback(s: &String) -> &String {
            // In this case, string is not allocated in the function.
            // So no dungling happens
            s
        }

        /*
        // This does not work because cloned data is owned by the function
        fn cloneback(s: &String) -> &String {
            &(s.clone())
        }*/

        let mut s = String::from("hello");
        // Passes mutable reference and borrowed back.
        // Interesting.. `ss` does not need `mut` for declaration.
        // That's because the compiler can infer mutability from return type of the function?
        let ss = borrowback2(&mut s);
        ss.push_str(", world-2!");
        println!("Mutated and borrowed back: {}", ss);

        fn borrowback2(s: &mut String) -> &mut String {
            s.push_str(", world-1!");
            s
        }
    }

    {
        let mut a = 1;
        change(&mut a);

        fn change(a: &mut i32) {
            // `+=` operator does not work for mutable reference. Interesting..
            //a += 1;
        }
    }
}
