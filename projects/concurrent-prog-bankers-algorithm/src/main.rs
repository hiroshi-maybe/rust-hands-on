mod banker;

use banker::Banker;
use std::{
    sync::{Arc, Mutex},
    thread,
};

const NUM_LOOP: usize = 1_000;

fn main() {
    let banker = Banker::<2, 2>::new([1, 1], [[1, 1], [1, 1]]);
    let banker0 = banker.clone();
    let banker1 = banker.clone();

    let current_eater = Arc::new(Mutex::new(0));
    let current_eater0 = current_eater.clone();
    let current_eater1 = current_eater.clone();

    let philosopher0 = thread::spawn(move || {
        for i in 0..NUM_LOOP {
            while !banker0.take(0, 0) {}
            while !banker0.take(0, 1) {}

            let mut current = current_eater0.lock().unwrap();
            if *current != 0 {
                println!("Switched to 0: {}-th eating", i);
                *current = 0;
            }

            banker0.release(0, 0);
            banker0.release(0, 1);
        }
        println!("0: finished eating");
    });

    let philosopher1 = thread::spawn(move || {
        for i in 0..NUM_LOOP {
            while !banker1.take(1, 0) {}
            while !banker1.take(1, 1) {}

            let mut current = current_eater1.lock().unwrap();
            if *current != 1 {
                println!("Switched to 1: {}-th eating", i);
                *current = 1;
            }

            banker1.release(1, 0);
            banker1.release(1, 1);
        }
        println!("1: finished eating");
    });

    philosopher0.join().unwrap();
    philosopher1.join().unwrap();
}
