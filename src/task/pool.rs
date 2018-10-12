use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Debug)]
struct ThreadInfo {
    id: i32,
    busy: AtomicBool,
}

pub struct PoolsThread<F: FnOnce() + Send> {
    thread: Arc<ThreadInfo>,
    task_sender: Sender<Box<F>>,
}

pub struct Pool<F: FnOnce() + Send> {
    threads: Vec<PoolsThread<F>>
}

fn handler<F>(rx: Receiver<Box<F>>, info: &mut Arc<ThreadInfo>)
    where F: FnOnce() + Send
{
    rx.recv().map(|f| {
        info.busy.store(true, Ordering::SeqCst);
        println!("#{} is busy", info.id);
        (f)();
        info.busy.store(false, Ordering::SeqCst);
        println!("#{} is free", info.id);
    }).unwrap();
}

fn init_thread<F>(n: i32) -> PoolsThread<F>
    where F: FnOnce() + Send + 'static
{
    println!("initiating {} thread in the pool", n);
    let (tx, rx): (Sender<Box<F>>, Receiver<Box<F>>) = channel();
    let pt = PoolsThread {
        thread: Arc::new(ThreadInfo {
            id: n,
            busy: AtomicBool::new(false),
        }),
        task_sender: tx,
    };
    let mut inner_info = Arc::clone(&pt.thread);
    thread::spawn(move || handler(rx, &mut inner_info));
    pt
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
//    p.threads.iter().for_each(|t| println!("{:#?}", t.thread));

    let free_thread = p.threads.iter()
        .filter(|t| t.thread.busy.load(Ordering::SeqCst) == false)
        .next().expect("There is no free threads!");
    println!("Executing task on thread #{}", free_thread.thread.id);
    free_thread.task_sender.send(f).expect("Unable to run the task");
}