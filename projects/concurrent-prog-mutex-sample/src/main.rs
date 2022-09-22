use std::{
    sync::{Arc, Mutex},
    thread,
};

/// https://www.oreilly.co.jp/books/9784873119595/
/// Ch 3.8.1

fn f(thread_id: u8, c: Arc<Mutex<u64>>) {
    loop {
        let mut val = c.lock().unwrap();

        if *val >= 1000 {
            break;
        }

        *val += 1;
        println!("{}: {}", thread_id, *val);
    }
}

// fn f_no_mutex(thread_id: u8, c: Arc<u64>) {
//     loop {
//         *c += 1; // mutability is not supported in Arc by default
//         println!("{}: {}", thread_id, *c);
//     }
// }

fn main() {
    let c0 = Arc::new(Mutex::new(0));
    let c1 = c0.clone();

    let t0 = thread::spawn(move || {
        f(0, c0);
    });
    let t1 = thread::spawn(move || {
        f(1, c1);
    });

    t0.join().unwrap();
    t1.join().unwrap();
}
