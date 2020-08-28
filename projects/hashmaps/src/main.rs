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
}
