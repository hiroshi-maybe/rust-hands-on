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
    }
}
