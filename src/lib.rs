use std::{
    sync::{mpsc, Arc, Mutex},
    thread, usize,
};

use serde::{Deserialize, Serialize};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[derive(Debug, Serialize, Deserialize)]
pub enum MsgType {
    Identify = 1,
    List,
    Relay,
    Server,
}

impl MsgType {
    fn from_u8(val: u8) -> Option<MsgType> {
        info!("val: {}", val);
        // TODO: drop connection if val is not a valid MsgType
        match val {
            48 => Some(MsgType::Identify),
            49 => Some(MsgType::List),
            50 => Some(MsgType::Relay),
            51 => Some(MsgType::Server),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub msg_type: MsgType,
    pub body: String,
}

impl Message {
    pub fn serialize(self) {
        let serialized = serde_json::to_string(&self);
    }

    pub fn deserialize(msg: &[u8]) -> Message {
        let msg_type = MsgType::from_u8(msg[0]).unwrap();
        let message = String::from_utf8(msg[1..].to_vec()).unwrap();

        info!("{}", message);

        Message {
            msg_type,
            body: message,
        }
    }
}

pub struct WorkerQueue {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl WorkerQueue {
    pub fn new(size: usize) -> WorkerQueue {
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        WorkerQueue {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    info!("Worker {} got a job; executing.", id);
                    job();
                }
                Err(_) => {
                    info!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id: id,
            thread: Some(thread),
        }
    }
}
