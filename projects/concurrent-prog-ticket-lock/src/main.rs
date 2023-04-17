use std::sync::Arc;

mod ticket_lock;

const NUM_LOOP: usize = 100_000;
const NUM_THREADS: usize = 4;

fn main() {
    let lock = Arc::new(ticket_lock::TicketLock::new(0));
    let mut v = Vec::new();

    for _ in 0..NUM_THREADS {
        let lock = lock.clone();
        let t = std::thread::spawn(move || {
            for _ in 0..NUM_LOOP {
                let mut data = lock.lock();
                *data += 1;
            }
        });
        v.push(t);
    }

    for t in v {
        t.join().unwrap();
    }

    println!(
        "COUNT = {} (expected = {})",
        *lock.lock(),
        NUM_LOOP * NUM_THREADS
    );
}
