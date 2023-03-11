/// References
/// * https://cfsamson.gitbook.io/green-threads-explained-in-200-lines-of-rust/
mod green;

const STACK_SIZE: usize = 2 * 1024 * 1024;

fn mash() {
    green::spawn(ortega, STACK_SIZE);
    for _ in 0..10 {
        println!("Mash!");
        green::schedule();
    }
}

fn ortega() {
    for _ in 0..10 {
        println!("Ortega!");
        green::schedule();
    }
}

fn gaia() {
    println!("Gaia called");
    green::spawn(mash, STACK_SIZE);
    for _ in 0..10 {
        println!("Gaia!");
        green::schedule();
    }
}

fn main() {
    println!("main called");
    green::spawn_from_main(gaia, STACK_SIZE);
}
