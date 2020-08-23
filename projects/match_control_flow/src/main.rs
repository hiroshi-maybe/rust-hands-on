fn main() {
    #[derive(Debug)]
    enum UsState {
        Alabama,
        Alaska,
    }

    #[derive(Debug)]
    enum Coin {
        Penny,
        Nickel,
        Dime,
        Quarter(UsState),
    }

    fn value(coin: &Coin) -> u8 {
        match coin {
            Coin::Penny => 1,
            Coin::Nickel => 5,
            Coin::Dime => 10,
            Coin::Quarter(_) => 25,
        }
    }

    {
        let coins: [Coin; 4] = [
            Coin::Penny, Coin::Nickel, Coin::Dime,
            Coin::Quarter(UsState::Alaska)];
        for c in coins.iter() {
            println!("value for {:?} is {}", c, value(&c));
        }
    }

    {
        fn plus_one(x: Option<i32>) -> Option<i32> {
            match x {
            None => None,
            Some(i) => Some(i+1),
            }
        }

        let five = Some(5);
        let six = plus_one(five);
        let none = plus_one(None);
        println!("{:?} + 1 = {:?}", five, six);
        println!("{:?} + 1 = {:?}", None as Option<i32>, none);
    }

    {
        let v = Some(3);
        if let Some(3) = v {
            println!("three");
        }
    }
}
