use std::thread;
use std::time::Duration;
use std::sync::mpsc::{self, Sender};

use std::sync::{Mutex, Arc};

fn main() {
    println!("** 16-1 Using Threads to Run Code Simultaneously");

    {
        // Creating a New Thread with spawn

        let handle = thread::spawn(|| {
            for i in 1..10 {
                println!("hi number {} from the spawned thread!", i);
                thread::sleep(Duration::from_millis(1));
            }
        });

        for i in 1..5 {
            println!("hi number {} from the main thread!", i);
            thread::sleep(Duration::from_millis(1));
        }

        handle.join().unwrap();

        println!("The spawned thread joined");
    }

    {
        // Using move Closures with Threads

        let v = vec![1, 2, 3];
        let handle = thread::spawn(move || {
            println!("Here's a moved vector in a new thread: {:?}", v);
        });

        // Throws an error because `v` is moved above
        // println!("{:?}", v);

        handle.join().unwrap();
    }

    println!("** 16-2 Using Message Passing to Transfer Data Between Threads");

    {
        // Using Message Passing to Transfer Data Between Threads

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let val = String::from("hi");
            tx.send(val).unwrap();
            // Error: `borrow of moved value: `val``
            // println!("val is {}", val);
        });

        let received = rx.recv().unwrap();
        println!("Got: {}", received);
    }

    {
        // Creating Multiple Producers by Cloning the Transmitter

        let (tx, rx) = mpsc::channel();

        fn send(tx: Sender<String>, vals: Vec<String>) {
            thread::spawn(move || {
                for val in vals {
                    tx.send(val).unwrap();
                    thread::sleep(Duration::from_secs(1));
                }
            });
        }

        {
            let vals = vec![
                String::from("hi"),
                String::from("from"),
                String::from("the"),
                String::from("thread"),
            ];
            let tx1 = Sender::clone(&tx);
            send(tx1, vals);
        }

        {
            let vals = vec![
                String::from("more"),
                String::from("messages"),
                String::from("for"),
                String::from("you"),
            ];
            send(tx, vals);
        }

        for received in rx {
            println!("Got: {}", received);
        }
    }

    println!("** 16-3 Using Mutexes to Allow Access to Data from One Thread at a Time");

    {
        // Using Mutexes to Allow Access to Data from One Thread at a Time

        let m = Mutex::new(5);

        {
            let mut num = m.lock().unwrap();
            *num = 6;
        }

        println!("m = {:?}", m);
    }

    {
        // Atomic Reference Counting with Arc<T>

        let counter = Arc::new(Mutex::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                let mut num = counter.lock().unwrap();
                *num += 1;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        println!("Result: {}", *counter.lock().unwrap());
    }

    //#[cfg(feature = "deadlock_detection")]
    {
        /*
        use parking_lot::deadlock;
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                let deadlocks = deadlock::check_deadlock();
                if deadlocks.is_empty() {
                    println!("No deadlock found");
                    continue;
                }

                println!("{} deadlocks detected", deadlocks.len());
                for (i, threads) in deadlocks.iter().enumerate() {
                    println!("Deadlock #{}", i);
                    for t in threads {
                        println!("Thread Id {:#?}", t.thread_id());
                        println!("{:#?}", t.backtrace());
                    }
                }
            }
        });*/

        // Code which causes a deadlock!!

        let counter1 = Mutex::new(0);

        println!("c1 will be locked");
        let mut locked_c1 = counter1.lock().unwrap();
        *locked_c1 += 1;
        println!("c1 is {}", locked_c1);

        // Without dropping the lock before locking the same mutex in the same thread again, it causes a deadlock.
        // drop(locked_c1);
        // println!("c1 is dropped before taking a lock again");

        println!("c1 will be locked again in the same thread");
        println!("c1: {}", counter1.lock().unwrap());
    }
}
