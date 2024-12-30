use std::{
    sync::{Arc, RwLock},
    thread,
};

#[allow(dead_code)]
fn deadlock_example1(val: Arc<RwLock<bool>>) {
    // Deadlock!
    let flag = val.read().unwrap();
    if *flag {
        *val.write().unwrap() = false;
        println!("flag is true");
    }

    // No deadlock!
    // let flag = *val.read().unwrap();
    // if flag {
    //     *val.write().unwrap() = false;
    //     println!("flag is true");
    // }
}

#[allow(dead_code)]
fn deadlock_example2(val: Arc<RwLock<bool>>) {
    // Deadlock!
    // let _flag = val.read().unwrap();
    // *val.write().unwrap() = false;
    // println!("deadlock");

    // No deadlock!
    let _ = val.read().unwrap();
    *val.write().unwrap() = false;
    println!("no deadlock");
}

fn main() {
    let val = Arc::new(RwLock::new(true));

    let t = thread::spawn(move || {
        // deadlock_example1(val);
        deadlock_example2(val);
    });

    t.join().unwrap();
}
