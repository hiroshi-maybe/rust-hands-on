use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use concurrent_prog_semaphore_sample::semaphore::Semaphore;

/// https://www.oreilly.co.jp/books/9784873119595/
/// Ch 3.8.5

const NUM_LOOP: usize = 1_000;
const NUM_THREADS: usize = 8;
const NUM_SEM: isize = 4;

static mut CNT: AtomicUsize = AtomicUsize::new(0);

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

fn main() {
    semapho_demo();
}
