pub mod pool {
    use std::sync::mpsc::{channel, Sender, Receiver};
    use std::thread;
    use std::thread::JoinHandle;

    pub struct PoolsThread {
        id: i32,
        busy: bool,
        task_sender: Sender<Box<FnOnce() + Send>>,
        handle: Box<JoinHandle<()>>,
    }

    pub struct Pool {
        threads: Vec<PoolsThread>
    }

    impl Pool {

        fn init_thread(n: i32) -> PoolsThread {
            println!("initiating {} thread in the pool", n);
            let mut busy: bool = false;
            let (tx, rx): (Sender<Box<FnOnce() + Send + 'static>>, Receiver<Box<FnOnce() + Send + 'static>>) = channel();
            let handle = Box::new(thread::spawn( move || {
                loop {
                    match rx.recv() {
                        Ok(task) => {
                            busy = true;
                            task();
                            busy = false;
                        }
                        _ => {}
                    }
                }
            }));
            PoolsThread{
                id: n,
                busy,
                task_sender: tx,
                handle
            }
        }

        pub fn new(count: i32) -> Self {
            Pool {
                threads: (0..count).map(|n| Self::init_thread(n)).collect()
            }
        }

        pub fn exec<F>(&self, f: Box<F>)
        where F: 'static + Send + FnOnce()
        {
            let free_thread = self.threads.iter()
                .filter(|t| t.busy == false)
                .next().expect("There is no free threads!");
            free_thread.task_sender.send(f);
        }
    }
}