use std::{sync::{RwLock, Arc}, thread};

/// https://www.oreilly.co.jp/books/9784873119595/
/// Ch 3.8.3

fn reader(lock: Arc<RwLock<i32>>) {
    loop {
        let v = lock.read().unwrap();
        println!("reader: {}", v);
        if *v >= 100 {
            break;
        }
    }
}

fn writer(lock: Arc<RwLock<i32>>) {
    loop {
        let mut v = lock.write().unwrap();
        *v += 1;
        println!("writer: {}", v);
        if *v >= 100 {
            break;
        }
    }
}

#[allow(dead_code)]
fn simple_sample() {
    let lock = RwLock::new(10);
    {
        let v1 = lock.read().unwrap();
        let v2 = lock.read().unwrap();
        println!("v1 = {}, v2 = {}", v1, v2);
    }

    {
        let mut v = lock.write().unwrap();
        *v = 7;
        println!("v = {}", v);
    }
}

fn main() {
    // simple_sample();

    let lock0 = Arc::new(RwLock::new(0));
    let lock1 = lock0.clone();

    let t0 = thread::spawn(move || {
        reader(lock0);
    });
    let t1 = thread::spawn(move || {
        writer(lock1);
    });

    t0.join().unwrap();
    t1.join().unwrap();
}
