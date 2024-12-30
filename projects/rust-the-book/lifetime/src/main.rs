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
            // If string slice is given, no error is thrown..
            // That is probably because string literal has 'static lifetime.
            res = longest(s1, s2);
        }
        println!("{} is longer string", res);

        {
            // This works
            fn longest1<'a>(x: &str, y: &str) -> &'a str {
                "really long string"
            }

            // Throws an error because String `res` is dropped when it's returned
            /*
            fn longest2<'a>(x: &str, y: &str) -> &'a str {
                let res = String::from("really long string");
                res.as_str()
            }*/

            println!("{}", longest1("a", "b"));
        }
    }

    struct ImportantExcerpt<'a> {
        part: &'a str,
    }

    {
        // Lifetime Annotations in Struct Definitions

        /*
        let it;
        {
            let novel = String::from("Call me Ishmael. Some years ago...");
            // Below line throws an error because novel is borrowed by `it` which lives longer
            let first_sentence = novel.split('.').next().expect("Could not find a '.'");
            it = ImportantExcerpt {
                part: first_sentence
            };
        }
        println!("{}", it.part);
        */

        let novel = String::from("Call me Ishmael. Some years ago...");
        let first_sentence = novel.split('.').next().expect("Could not find a '.'");
        let it = ImportantExcerpt {
            part: first_sentence,
        };
        println!("{}", it.part);
    }

    let it = ImportantExcerpt { part: "abc" };

    {
        // Lifetime Annotations in Method Definitions

        impl<'b> ImportantExcerpt<'b> {
            fn level(&self) -> i32 {
                3
            }

            fn announce_and_return_part(&self, announcement: &str) -> &str {
                println!("Attention please; {}", announcement);
                self.part
            }
        }
        println!("Level is {}", it.level());
        println!("Part is {}", it.announce_and_return_part("foo"));
    }

    {
        use std::fmt::Display;
        fn not_longest_with_an_announcement<'a, 'b, T>(x: &'a str, y: &'b str, ann: T) -> &'a str
        where
            T: Display,
        {
            println!("Announcement: {}", ann);
            println!("{} is never returned", y);
            x
        }

        println!(
            "{} is returned",
            not_longest_with_an_announcement("xxx", "yyy", 12345)
        );
    }
}
