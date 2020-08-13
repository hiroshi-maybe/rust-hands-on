fn main() {
    {
        // `first_word` with size

        fn first_word(s: &String) -> usize {
            let bytes = s.as_bytes();
            for (i, &item) in bytes.iter().enumerate() {
                if item == b' ' {
                    return i;
                }
            }

            s.len()
        }

        let mut s = String::from("hello world");
        let word = first_word(&s);
        s.clear();

        println!("Position of the end of the first word is {}", word);
    }

    {
        // String Slices

        let s = String::from("hello world");
        let hello = &s[0..5];
        let world = &s[6..11];
        println!("s = \"{}\" + \"{}\"", hello, world);

        // Out of bounds panic
        //let world = &s[6..12];
        //println!("out of bounds:{}", world);

        let rld = &world[2..];
        let wo = &world[..2];
        let wo2 = &wo[..];
        println!("wo = \"{}\" = \"{}\", rld = \"{}\"", wo, wo2, rld);
    }

    {
        // `first_word` with string slice

        let mut s = String::from("hello world");
        let w = first_word(&s);

        // This throws an error because `w` from borrowed `s` is referred after mutable operation
        //s.clear();

        println!("first word is: {}", w);

        // This works because the life of `w` ended.
        s.clear();

        println!("cleared: {}", s);

        // with &str parameter type, string literal works too!
        let s = "hello world";
        let w = first_word(s);
        println!("first word is {}", w);

        fn first_word(s: &str) -> &str {
            let bytes = s.as_bytes();
            for (i, &item) in bytes.iter().enumerate() {
                if item == b' ' {
                    return &s[0..i];
                }
            }

            &s[..]
        }
    }
}
