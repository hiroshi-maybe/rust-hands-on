use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce(usize) + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(usize) + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            // This is non-blocking
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("[Worker {}] start shutting down", worker.id);
            if let Some(thread) = worker.thread.take() {
                // This is blocking. Wait until terminate signal is received.
                thread.join().unwrap();
            }

            println!("[Worker {}] finish shutting down", worker.id);
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Four states
    ///
    /// 1. waiting for a lock. Go to state #2 once a lock is obtained.
    /// 2. waiting for a message with a mutex lock
    ///     ¡) message is a new job => go to state #3
    ///     ii) message is terminate => go to state #4
    /// 3. processing a request (lock is released), back to state #1 once completed.
    /// 4. terminated (handled by join)
    ///
    /// State #2 is killed first with a terminate message
    /// State #1 and #3 take a lock and are killed with succeeding terminate messages
    ///
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message: Message;
            {
                println!("[Worker {}] waiting for a lock", id);
                // lock() acquires the mutex
                let receiver = receiver.lock().unwrap();
                println!("[Worker {}] got a lock", id);
                // recv() blocks until a job becomes available
                message = receiver.recv().unwrap();
                println!("[Worker {}] releasing a lock", id);
            }
            // Receiver is dropped -> Mutex is freed -> Other thread takes a lock

            //let job = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("[Worker {}] got a job; executing.", id);
                    job(id);
                    println!("[Worker {}] finished.", id);
                }
                Message::Terminate => {
                    println!("[Worker {}] received a terminate signal.", id);
                    break;
                }
            }
        });
        println!("[Worker {}] Thread spawned", id);

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
