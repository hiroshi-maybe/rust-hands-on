use std::{thread, sync::{Barrier, Arc}};

/// https://www.oreilly.co.jp/books/9784873119595/
/// Ch 3.8.4

fn main() {
    let mut ts = vec![];
    let barrier = Arc::new(Barrier::new(10));

    for tid in 0..20 {
        let b = barrier.clone();
        let t = thread::spawn(move || {
            b.wait();
            println!("{} finished barrier", tid);
        });
        ts.push(t);
    }

    for t in ts {
        t.join().unwrap();
    }
}
