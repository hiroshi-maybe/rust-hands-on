use std::{
    sync::{Arc, Mutex},
    thread,
};

fn main() {
    let c0 = Arc::new(Mutex::new(()));
    let c1 = Arc::new(Mutex::new(()));

    let c_p0_left = c0.clone();
    let c_p0_right = c1.clone();

    let c_p1_left = c1.clone();
    let c_p1_right = c0.clone();

    let p0 = thread::spawn(move || {
        for i in 0..100_000 {
            let _n1 = c_p0_left.lock().unwrap();
            // println!("0: {}-th left", i);
            let _n2 = c_p0_right.lock().unwrap();
            println!("0: {}-th right and eating", i);
        }
    });

    let p1 = thread::spawn(move || {
        for i in 0..100_000 {
            let _n1 = c_p1_left.lock().unwrap();
            // println!("1: {}-th left", i);
            let _n2 = c_p1_right.lock().unwrap();
            println!("1: {}-th right and eating", i);
        }
    });

    p0.join().unwrap();
    p1.join().unwrap();
}
