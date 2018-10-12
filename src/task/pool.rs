pub mod pool {
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::thread;

    type F = FnOnce() + Send;

    pub struct PoolsThread {
        id: i32,
        busy: bool,
        task_sender: Sender<Box<F>>,
    }

    pub struct Pool {
        threads: Vec<PoolsThread>
    }


    fn init_thread(n: i32) -> PoolsThread {
        println!("initiating {} thread in the pool", n);
//        let mut busy: bool = false;
        let (tx, rx): (Sender<Box<F>>, Receiver<Box<F>>) = channel();
        thread::spawn(move || rx.recv().map(|f| (f)()).unwrap());

        PoolsThread {
            id: n,
            busy: false,
            task_sender: tx,
        }
    }


    pub fn new(count: i32) -> Pool {
        Pool {
            threads: (0..count).map(|n| init_thread(n)).collect()
        }
    }

    pub fn exec<F>(p: &Pool, _f: Box<F>)
        where F: FnOnce() + Send + 'static
    {
        let free_thread = p.threads.iter()
            .filter(|t| t.busy == false)
            .next().expect("There is no free threads!");
        println!("Executing task on thread #{}", free_thread.id);
        let f = Box::new(|| println!("yo"));
        free_thread.task_sender.send(f).expect("Unable to run the task");
    }
}