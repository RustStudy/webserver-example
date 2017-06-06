use std::thread;
// 使用channel来进行线程通信
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

enum Message {
    NewJob(Job),
    Terminate,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        // 一般不允许Box<T>中移出T，因为不知道T的大小
        // 但是我们可以使用self：Box <Self>以获取自身的所有权，并将值移出Box <T>
        // 此时任何FnOnce()闭包都可以使用此call_box方法
        // Rust还在改进，将来就不需要这样写，完全可以直接调用闭包，不过也是将来的事了- 2017.06.06 nightly 1.18
        (*self)()
    }
}

// 该类型别名用于保存接收的闭包类型
type Job = Box<FnBox + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);

                        job.call_box();
                    },
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", id);

                        break;
                    },
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}


impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    // try : cargo doc --open 来生成注释文档
    pub fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        //  因为Rust channel是multiple producer, single consumer模式
        // 所以我们用线程安全的方式来共享receiver的方法来让多个worker实例使用receiver
        // Arc<Mutex<T>>
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
           workers.push(Worker::new(id, receiver.clone()));
        }

        ThreadPool {
            workers,
            sender,
        }
    }

    // 参考spawn的实现，complier drive development
    // pub fn spawn<F, T>(f: F) -> JoinHandle<T>
    //     where
    //         F: FnOnce() -> T + Send + 'static,
    //         T: Send + 'static
    // execute对于每个请求来说我们仅仅需要执行一次，所以这里需要FnOnce来获取self就够了
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        // 把job这个trait object发送给worker
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
