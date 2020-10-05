use add_one;
use rand;

// $ cargo build
// $ cargo run -p adder

fn main() {
    let num = 10;
    println!("Hello, world! {} plus one is {}!", num, add_one::add_one(num));
}
