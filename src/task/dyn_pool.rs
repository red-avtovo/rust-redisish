use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use uuid::Uuid;

#[derive(Debug)]
struct ThreadInfo {
    id: Uuid,
    busy: AtomicBool,
}

pub struct PooledThread<F: FnOnce() + Send> {
    thread: Arc<ThreadInfo>,
    task_sender: Sender<Box<F>>,
}

pub struct Pool<F: FnOnce() + Send> {
    max_count: usize,
    idle_threads: usize,
    threads: Vec<PooledThread<F>>,
}

fn handler<F>(rx: Receiver<Box<F>>, info: &mut Arc<ThreadInfo>)
    where F: FnOnce() + Send
{
    loop {
        match rx.recv() {
            Ok(f) => {
                info.busy.store(true, Ordering::SeqCst);
                println!("#{} is busy", info.id);
                (f)();
                info.busy.store(false, Ordering::SeqCst);
                println!("#{} is free", info.id);
            }
            _ => return
        }
    }
}

fn init_thread<F>() -> PooledThread<F>
    where F: FnOnce() + Send + 'static
{
    let (tx, rx): (Sender<Box<F>>, Receiver<Box<F>>) = channel();
    let pt = PooledThread {
        thread: Arc::new(ThreadInfo {
            id: Uuid::new_v4(),
            busy: AtomicBool::new(false),
        }),
        task_sender: tx,
    };
//    println!("initiating [{}] thread in the pool", pt.thread.id);
    let mut inner_info = Arc::clone(&pt.thread);
    thread::spawn(move || handler(rx, &mut inner_info));
    pt
}


pub fn new<F>(max_count: usize, idle_threads: usize) -> Pool<F>
    where F: FnOnce() + Send + 'static
{
    Pool {
        max_count,
        idle_threads,
        threads: (0..idle_threads).map(|_i| init_thread()).collect(),
    }
}

fn pre_exec<F>(p: &mut Pool<F>)
    where F: FnOnce() + Send + 'static
{
    draw_thread(p);
    let threads = p.threads.len();
    if p.max_count <= threads { return; }
    p.threads.push(init_thread());
    draw_thread(p);
}

fn post_exec<F>(p: &mut Pool<F>)
    where F: FnOnce() + Send + 'static
{
    draw_thread(&p);
    let free_threads = p.threads.iter_mut().enumerate()
        .filter(|(_i, t)| t.thread.busy.load(Ordering::SeqCst) == false)
        .map(|(i, _t)| i)
        .collect::<Vec<_>>();
    let free_threads_count = free_threads.len();

    if free_threads_count > p.idle_threads { return; }
    let threads_to_delete = p.idle_threads - free_threads_count;
    let _ = free_threads.iter().take(threads_to_delete)
        .map(|i| p.threads.remove(*i));
    draw_thread(&p);
}

fn exec_task<F>(threads: &Vec<PooledThread<F>>, f: Box<F>)
    where F: FnOnce() + Send + 'static
{
    let free_thread = threads.iter()
        .filter(|t| t.thread.busy.load(Ordering::SeqCst) == false)
        .next().expect("There is no free threads!");
//    println!("Executing task on thread {}", free_thread.thread.id);
    free_thread.task_sender.send(f).expect("Unable to run the task");
}

fn draw_thread<F>(p: &Pool<F>)
    where F: FnOnce() + Send + 'static
{
    let n = p.threads.len();
    let thread_view: String = p.threads.iter().map(|t: &PooledThread<F>| &t.thread.busy )
        .map(|busy_state| match busy_state.load(Ordering::SeqCst) {
            true => "O",
            false => "_"
        })
        .collect::<Vec<_>>().join(", ");

    println!("{} [{}]", n, thread_view)
}

pub fn exec<F>(p: &mut Pool<F>, f: Box<F>)
    where F: FnOnce() + Send + 'static
{
    pre_exec(p);
    exec_task(&p.threads, f);
    post_exec(p);
}