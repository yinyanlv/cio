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

enum Message {
    NewTask(Task),
    Terminate
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>
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

        println!("send message: new task");
        self.sender.send(Message::NewTask(task)).unwrap();
    }
}

impl Drop for ThreadPool {

    fn drop(&mut self) {

        for _ in &mut self.workers {

            println!("send message: terminate");

            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {

            println!("join and drop work {}", worker.id);

            if let Some(thread) = worker.thread.take() {

                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {

    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        
        let thread = thread::spawn(move || {

            loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    
                    Message::Terminate => {

                        println!("Worker {} receive message: terminate", id);
                        break;
                    },

                    Message::NewTask(task) => {

                        println!("Worker {} receive message: new task", id);
                        task.call();
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread)
        }
    }
}
