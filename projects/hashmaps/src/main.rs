use std::collections::HashMap;

fn main() {
    {
        let mut scores = HashMap::new();

        let blue = String::from("Blue");

        scores.insert(blue, 10);
        scores.insert(String::from("Yellow"), 50);
        // This throws an error because `blue` is moved to the HashMap
        // println!("{:?}", blue);
        println!("{:?}", scores);
    }

    {
        let teams = vec!["Blue", "Yellow"];
        let initial_scores = vec![10, 50];

        let scores1: HashMap<_,_> = teams.into_iter()
            .zip(initial_scores.into_iter())
            .collect();

        for (key, value) in &scores1 {
            println!("{} -> {}", key, value);
        }
        println!("{:?}", scores1);

        for (key, value) in scores1 {
            println!("{} -> {}", key, value);
        }
        // Moved above
        //println!("{:?}", scores1);

        let teams2 = vec!["Blue", "Yellow"];
        let initial_scores2 = vec![10, 10];

        let scores2: HashMap<_,_> = initial_scores2.into_iter()
            .zip(teams2.into_iter())
            .collect();

        println!("10 -> {:?}", scores2.get(&10));
        println!("20 -> {:?}", scores2.get(&20));
        for (key, value) in &scores2 {
            println!("{} -> {}", key, value);
        }
    }

    {
        let mut scores = HashMap::new();
        scores.insert("Blue", 10);
        scores.insert("Blue", 11);
        println!("{:?}", scores);

        let b = scores.entry("Blue").or_insert(100);
        println!("{}", b);
        let y = scores.entry("Yellow").or_insert(100);
        println!("{}", y);

        println!("{:?}", scores);
    }

    {
        let text = "hello world wonderful world";
        let mut map = HashMap::new();
        let mut a=0;
        for word in text.split_whitespace() {
            let cnt = map.entry(word).or_insert(0);
            *cnt += 1;

            // Possible to pass dereferenced copyable value
            a = *cnt;
        }
        println!("{}", a);
        println!("{:?}", map);
    }

    {
        #[derive(Debug)]
        struct Counter {
            val: i32,
        };

        let text = "hello world wonderful world";
        let mut map = HashMap::new();
        let mut a=Counter { val: 0 };
        for word in text.split_whitespace() {
            let cnt = map.entry(word).or_insert(Counter { val: 0 });
            cnt.val += 1;
            // Error because mutable reference cannot be borrowed without move
            //a = *cnt;
        }
        println!("{:?}", a);
        println!("{:?}", map);
    }
}
