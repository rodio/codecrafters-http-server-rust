use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

struct Worker {}

type Job = Box<dyn FnOnce() + Send>;

pub struct ThreadPool {
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<Job>();

        let receiver = Arc::new(Mutex::new(receiver));

        let size = 20;
        for _ in 0..size {
            Worker::new(Arc::clone(&receiver));
        }

        ThreadPool { sender }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Worker {
    fn new(reciever: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        thread::spawn(move || loop {
            let message = reciever.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("worker is executing a job");
                    job();
                }
                Err(_) => break,
            }

            //let job_guard = reciever.lock();
            //match job_guard {
            //    Ok(job_guard) => match job_guard.recv() {
            //        Ok(job) => {
            //            println!("worker is executing a job");
            //            job();
            //        }
            //        Err(e) => {
            //            println!("worker is shutting down: {}", e);
            //            break;
            //        }
            //    },
            //    Err(e) => {
            //        // can happen if another thread panicked
            //        println!("worker can't acquire lock: {}", e);
            //        break;
            //    }
            //}
        });

        Worker {}
    }
}
