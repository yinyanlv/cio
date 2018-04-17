use std::thread;
use std::sync::{mpsc, Arc, Mutex};

trait FnBox {
    fn call(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call(self: Box<Self>) {
        (*self)();
    }
}

type Task = Box<FnBox + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Task>
}

impl ThreadPool {

    pub fn new(size: usize) -> ThreadPool {

        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender
        }
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {

        let task = Box::new(f);

        self.sender.send(task).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>
}

impl Worker {

    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Task>>>) -> Worker {
        
        let thread = thread::spawn(move || {

            loop {
                let task = receiver.lock().unwrap().recv().unwrap();

                println!("Worker {}; executing.", id);
                
                task.call();
            }
        });

        Worker {
            id,
            thread
        }
    }
}
