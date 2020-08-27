fn main() {
    {
        // creating a new String
        let data = "initial contents";
        let s1 = data.to_string();
        let s2 = "initial contents".to_string();
        let s3 = String::from("initial contents");

        println!("\"{}\" = \"{}\" = \"{}\"? {}", s1, s2, s3, s1==s2 && s2==s3);
    }

    {
        // Updating a String
        let mut s1 = String::from("foo");
        s1.push_str("bar");
        println!("{}", s1);

        let mut s1 = String::from("lo");
        s1.push('l');
        println!("{}", s1);

        let s1 = String::from("Hello, ");
        let mut s2 = String::from("world!");
        let s3 = s1 + &s2 + " helloo";
        // throws an error because s1 is moved to s3
        //println!("{}", s1);
        println!("{}", s3);

        s2.push_str(" world!");
        println!("{}", s2);

        let mut s1 = String::from("tic");
        let s2 = "tac";
        let s3 = "toe".to_string();
        let s = format!("{}-{}-{}", s1, s2, s3);

        // This works because format!() does not take the ownership
        s1.push_str("tic");

        println!("{}, {}", s, s1);
    }

    {
        // 4 bytes, 4 unicode characters
        let hello1 = String::from("Hola");
        // 24 bytes, 12 unicode chars
        let hello2 = String::from("Здравствуйте");
        // This works because it's character boundary
        println!("{}", &hello2[0..4]);
        // Panic due to invalid char boundary error
        //println!("{}", &hello2[0..1]);

        // 25 bytes, 7 unicode chars
        let emoji = String::from("👨‍👩‍👧‍👧");
        // 3 bytes, 1 unicode char
        let ga = String::from("あ");
        let strs = [hello1, hello2, emoji, ga];
        for s in &strs {
            println!("{} is {} bytes, {} unicode chars", s, s.len(), s.chars().count());
        }
    }
}
