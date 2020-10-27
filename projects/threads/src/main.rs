use std::thread;
use std::time::Duration;

fn main() {
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
}
