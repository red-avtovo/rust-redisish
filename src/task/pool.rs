use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

//type F = FnOnce() + Send;

pub struct PoolsThread<F: FnOnce() + Send> {
    id: i32,
    busy: bool,
    task_sender: Sender<Box<F>>,
}

pub struct Pool<F: FnOnce() + Send> {
    threads: Vec<PoolsThread<F>>
}

fn handler<F>(rx: Receiver<Box<F>>)
    where F: FnOnce() + Send
{
    rx.recv().map(|f| (f)()).unwrap()
}

fn init_thread<F>(n: i32) -> PoolsThread<F>
    where F: FnOnce() + Send + 'static
{
    println!("initiating {} thread in the pool", n);
//        let mut busy: bool = false;
    let (tx, rx): (Sender<Box<F>>, Receiver<Box<F>>) = channel();
    thread::spawn(move || handler(rx));

    PoolsThread {
        id: n,
        busy: false,
        task_sender: tx,
    }
}


pub fn new<F>(count: i32) -> Pool<F>
where F: FnOnce() + Send + 'static
{
    Pool {
        threads: (0..count).map(|n| init_thread(n)).collect()
    }
}

pub fn exec<F>(p: &Pool<F>, f: Box<F>)
    where F: FnOnce() + Send
{
    let free_thread = p.threads.iter()
        .filter(|t| t.busy == false)
        .next().expect("There is no free threads!");
    println!("Executing task on thread #{}", free_thread.id);
    free_thread.task_sender.send(f).expect("Unable to run the task");
}