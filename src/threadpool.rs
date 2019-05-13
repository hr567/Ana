// FIXME: Use FnBox API for rust above 1.35
use std::sync;
use std::thread;

pub struct ThreadPool {
    task_sender: sync::mpsc::Sender<Box<dyn FnBox + Send + 'static>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "Thread pool must has as least 1 worker");

        let (worker_sender, worker_receiver) = sync::mpsc::channel::<Worker>();
        let (task_sender, task_receiver) = sync::mpsc::channel::<Box<dyn FnBox + Send + 'static>>();

        for _ in 0..size {
            worker_sender
                .send(Worker::new(worker_sender.clone()))
                .expect("Failed to send new thread worker");
        }

        thread::spawn(move || loop {
            for task in task_receiver.recv() {
                worker_receiver
                    .recv()
                    .expect("Failed to get new worker")
                    .spawn(task);
            }
        });

        ThreadPool {
            task_sender: task_sender,
        }
    }

    pub fn spawn(&self, task: impl FnOnce() + Send + 'static) {
        self.task_sender
            .send(Box::new(task))
            .expect("Failed to spawn new task");
    }
}

struct Worker {
    sender: sync::mpsc::Sender<Worker>,
}

impl Worker {
    fn new(sender: sync::mpsc::Sender<Worker>) -> Worker {
        Worker { sender }
    }

    fn spawn(self, func: Box<dyn FnBox + Send + 'static>) {
        thread::spawn(move || {
            func.call_box();
            self.sender
                .send(Worker::new(self.sender.clone()))
                .unwrap_or_default();
        });
    }
}

trait FnBox {
    fn call_box(self: Box<Self>) -> ();
}

impl<T> FnBox for T
where
    T: FnOnce(),
{
    fn call_box(self: Box<Self>) -> () {
        (*self)()
    }
}
