/// References
/// * https://cfsamson.gitbook.io/green-threads-explained-in-200-lines-of-rust/
mod green;

const STACK_SIZE: usize = 2 * 1024 * 1024;

#[allow(dead_code)]
fn mash() {
    // println!("Mash called");
    green::spawn(ortega, STACK_SIZE);
    // println!("Welcome back to Mash!");
    for _ in 0..10 {
        println!("Mash!");
        green::schedule();
    }
}

#[allow(dead_code)]
fn ortega() {
    for _ in 0..10 {
        println!("Ortega!");
        green::schedule();
    }
}

#[allow(dead_code)]
fn gaia() {
    // println!("Gaia called");
    green::spawn(mash, STACK_SIZE);
    // println!("Welcome back to Gaia!");
    for _ in 0..10 {
        println!("Gaia!");
        green::schedule();
    }
}

fn producer() {
    let id = green::spawn(consumer, STACK_SIZE);
    for i in 0..10 {
        green::send(id, i);
    }
}

fn consumer() {
    for _ in 0..10 {
        let msg = green::recv().unwrap();
        println!("received: count = {}", msg);
    }
}

fn main() {
    // green::spawn_from_main(gaia, STACK_SIZE);
    green::spawn_from_main(producer, STACK_SIZE);
}
