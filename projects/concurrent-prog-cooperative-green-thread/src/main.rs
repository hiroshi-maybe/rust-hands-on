/// References
/// * https://cfsamson.gitbook.io/green-threads-explained-in-200-lines-of-rust/
mod green;

fn mash() {
    green::spawn(ortega, 2 * 1024 * 1024);
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
    green::spawn(mash, 2 * 1024 * 1024);
    for _ in 0..10 {
        println!("Gaia!");
        green::schedule();
    }
}

fn main() {
    println!("main called");
    green::spawn_from_main(gaia, 2 * 1024 * 1024);
}
