fn main() {
    {
        // Preventing Dangling References with Lifetimes

        // lifetime of `r` starts
        let r;

        {
            // lifetime of `x` starts

            let x = 5;
            // Borrow doesn't work because 5 is dropped outside of the scope
            //r = &x;
            // Copy works
            r = x;

            // lifetime of `x` ends
        }

        println!("r: {}", r);

        // lifetime of `r` ends
    }

    {
        // Generic Lifetimes in Functions

        fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
            return if x.len() > y.len() { x } else { y };
        }

        /*
        let res;
        let s1 = String::from("abcd");

        {
            let s2 = String::from("xyz");
            // This is the error
            res = longest(s1.as_str(), s2.as_str());
        }

        println!("{} is longer string", res);
        */

        let res;
        let s1 = "abc";
        {
            let s2 = "wxyz";
            res = longest(s1, s2);
        }
        println!("{} is longer string", res);
    }
}
