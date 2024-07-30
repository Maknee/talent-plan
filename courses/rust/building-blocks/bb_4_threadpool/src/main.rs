use std::thread::JoinHandle;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use anyhow::Result;
use std::sync::{Arc, Mutex};

type Job = Box<dyn FnOnce() + Send + 'static>;

type Channel<T> = (Sender<T>, Receiver<T>);
#[derive(Debug, Default)]
struct ThreadPool {
    channels: Vec<Sender<Job>>,
    threads: Vec<JoinHandle<Result<()>>>,
    index: Arc<Mutex<usize>>,
}

impl ThreadPool {
  fn new(threads: u32) -> Result<Self> {
    let mut channels = Vec::with_capacity(threads as usize);
    let mut handles = Vec::with_capacity(threads as usize);

    for _ in 0..threads {
        let (tx, rx) = channel();

        let join_handle = thread::spawn(move || {
            loop {
                let f: Job = rx.recv()?;
                f();
            }
            Ok(())
        });
        channels.push(tx);
        handles.push(join_handle);
    }
    Ok(Self {
        channels: channels,
        threads: handles,
        index: Default::default(),
    })
  }

  fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static {
    self.channels[*self.index.lock().unwrap() % self.channels.len()].send(Box::new(job)).unwrap();
    *self.index.lock().unwrap() += 1;
  }

  fn wait(&mut self) {
    self.channels.clear();

    while let Some(j) = self.threads.pop() {
        j.join().unwrap();
    }
  }
}

fn main() -> Result<()> {
    let mut tp = ThreadPool::new(4)?;
    for i in 0..1000 {
        tp.spawn(move || {
            println!("HELLO! {i}");
        });
    }
    tp.wait();
    println!("Hello, world!");
    Ok(())
}
