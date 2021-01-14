use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
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
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {

            let job: Job;
            {
                println!("[Worker {}] waiting for a lock", id);
                // lock() acquires the mutex
                let receiver = receiver.lock().unwrap();
                println!("[Worker {}] got a lock", id);
                // recv() blocks until a job becomes available
                job = receiver.recv().unwrap();
            }
            // Receiver is dropped -> Mutex is freed -> Other thread takes a lock

            //let job = receiver.lock().unwrap().recv().unwrap();

            println!("[Worker {}] got a job; executing.", id);
            job(id);
            println!("[Worker {}] finished.", id);
        });
        println!("[Worker {}] Thread spawned", id);

        Worker { id, thread }
    }
}
