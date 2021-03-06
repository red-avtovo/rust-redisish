extern crate uuid;

use std::io;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

mod task;
mod redis;
use task::dyn_pool as pool;
use redis::ish::handle_client;

//echo "PUBLISH info one, info 2, Grüße, Jürgen ❤" | nc 127.0.0.1 8080
//echo "RETRIEVE" | nc localhost 8080

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let atomic_data: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let mut pool = pool::new(3,1);
    // accept connections and process them serially
    for stream in listener.incoming() {
        let arc = Arc::clone(&atomic_data);
        pool::exec(&mut pool,Box::new(move || handle_client(stream.unwrap(), &arc)))
    }
    Ok(())
}