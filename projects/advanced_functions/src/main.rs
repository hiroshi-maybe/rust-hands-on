fn main() {
    {
        // Function Pointers

        fn add_one(x: i32) -> i32 { x + 1 }
        let closure = |p| { p + 1 };

        fn do_twice(f: fn(i32) -> i32, arg: i32) -> i32 {
            f(arg) + f(arg)
        }

        // Passing a function pointer
        let ans = do_twice(add_one, 1);
        println!("The answer is: {}", ans);

        // Needs `Box` to avoid the following error
        // ```
        // the size for values of type `(dyn std::ops::Fn(i32) -> i32 + 'static)` cannot be known at compilation time
        // doesn't have a size known at compile-time
        // ```
        fn do_twice2(f: Box<dyn Fn(i32) -> i32>, arg: i32) -> i32 {
            f(arg) + f(arg)
        }

        let ans_c1 = do_twice2(Box::new(add_one), 1);
        let ans_c2 = do_twice2(Box::new(closure), 1);
        println!("The answer is: {} and {}", ans_c1, ans_c2);

        // Function pointers implement closure traits (`Fn`, `FnMut` and `FnOnce`).

        let ans1 = do_twice(closure, 1);
        println!("The answer is: {}", ans1);

        #[derive(Debug)]
        enum Status {
            Value(u32),
            Stop,
        }

        let ls: Vec<Status> = (0u32..20).map(Status::Value).collect();
        println!("{:?}", ls);
    }

    {
        // Returning Closures

        // Cannot compile without `Box<dyn {function type}>`. Otherwise the following error is thrown:
        // ```
        // return type cannot have an unboxed trait object
        // doesn't have a size known at compile-time
        // ```
        // fn returns_closure() -> dyn Fn(i32) -> i32 { |x| x + 1 }
        fn returns_closure() -> Box<dyn Fn(i32) -> i32> { Box::new(|x| x + 1) }

        let c = returns_closure();
        println!("From returned closure: {}", (*c)(1));
    }
}
