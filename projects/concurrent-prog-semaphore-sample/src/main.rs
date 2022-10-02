use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use concurrent_prog_semaphore_sample::channel;
use concurrent_prog_semaphore_sample::Semaphore;

/// https://www.oreilly.co.jp/books/9784873119595/
/// Ch 3.8.5

const NUM_LOOP: usize = 1_000;
const NUM_THREADS: usize = 8;
const NUM_SEM: isize = 4;

static mut CNT: AtomicUsize = AtomicUsize::new(0);

#[allow(dead_code)]
fn semapho_demo() {
    let sem = Arc::new(Semaphore::new(NUM_SEM));

    let mut ts = vec![];
    for i in 0..NUM_THREADS {
        let s = sem.clone();
        let t = thread::spawn(move || {
            for _ in 0..NUM_LOOP {
                s.wait();

                unsafe { CNT.fetch_add(1, Ordering::SeqCst) };
                let n = unsafe { CNT.load(Ordering::SeqCst) };
                println!("semaphore: i = {}, CNT = {}", i, n);
                assert!(n as isize <= NUM_SEM);
                unsafe { CNT.fetch_sub(1, Ordering::SeqCst) };

                s.post();
            }
        });
        ts.push(t);
    }

    for t in ts {
        t.join().unwrap();
    }
}

#[allow(dead_code)]
fn channel_demo() {
    let (tx, rx) = channel(4);
    let mut v = Vec::new();

    // receiver thread
    let t = std::thread::spawn(move || {
        let mut cnt = 0;
        while cnt < NUM_THREADS * NUM_LOOP {
            let n = rx.recv();
            println!("recv: n = {:?}", n);
            cnt += 1;
        }
    });

    v.push(t);

    // sender threads
    for i in 0..NUM_THREADS {
        let tx0 = tx.clone();
        let t = std::thread::spawn(move || {
            for j in 0..NUM_LOOP {
                tx0.send((i, j));
            }
        });
        v.push(t);
    }

    for t in v {
        t.join().unwrap();
    }
}

fn main() {
    // semapho_demo();
    channel_demo();
}
