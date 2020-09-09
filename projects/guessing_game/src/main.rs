use rand::Rng;
use std::io;
use std::cmp::Ordering;
mod guess;

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1,101);
    // println!("The secret number is: {}", secret_number);

    loop {
        println!("Please input your guess.");

        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let guess: i32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(e) => {
                println!("Encountered an error in parsing {}: {}", guess.trim(), e);
                continue;
            }
        };

        let guess = guess::Guess::new(guess);

        println!("You guessed: {}", guess.value());

        match guess.value().cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
