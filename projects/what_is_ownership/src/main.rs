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
}
