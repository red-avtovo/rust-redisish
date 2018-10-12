use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Debug)]
struct ThreadInfo {
    id: i32,
    busy: Arc<AtomicBool>,
}

pub struct PoolsThread<F: FnOnce() + Send> {
    thread: ThreadInfo,
    task_sender: Sender<Box<F>>,
}

pub struct Pool<F: FnOnce() + Send> {
    threads: Vec<PoolsThread<F>>
}

fn handler<F>(rx: Receiver<Box<F>>, busy: &mut Arc<AtomicBool>)
    where F: FnOnce() + Send
{
    rx.recv().map(|f| {
        busy.store(true, Ordering::SeqCst);
        println!("I'm busy");
        (f)();
        busy.store(false, Ordering::SeqCst);
        println!("I'm free");
    }).unwrap();
}

fn init_thread<F>(n: i32) -> PoolsThread<F>
    where F: FnOnce() + Send + 'static
{
    println!("initiating {} thread in the pool", n);
    let (tx, rx): (Sender<Box<F>>, Receiver<Box<F>>) = channel();
    let pt = PoolsThread {
        thread: ThreadInfo {
            id: n,
            busy: Arc::new(AtomicBool::new(false)),
        },
        task_sender: tx,
    };
    let mut inner_busy = Arc::clone(&pt.thread.busy);
    thread::spawn(move || handler(rx, &mut inner_busy));
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