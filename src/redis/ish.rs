use std::io::*;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
enum Command {
    PUBLISH(Vec<String>),
    RETRIEVE,
    QUIT,
    NONE,
}

macro_rules! log_if_error {
        ($e:expr) => {
            if let Err(e) = $e {
                eprint!("There is an error happened: {}", e)
            }
        }
    }

pub fn handle_client(stream: TcpStream, data: &Arc<Mutex<Vec<String>>>) {
    use self::Command::*;
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    loop {
        let mut buf = String::new();
        if let Err(_) = reader.read_line(&mut buf) { break; }

        let command = get_command(buf);
        println!("Received command: {:?}", command);
        match command {
            PUBLISH(strings) => {
                println!("publish: {:?}", strings);
                strings.iter().for_each(|s| data.lock().unwrap().push(s.clone()))
            }
            RETRIEVE => {
                match data.lock().unwrap().pop() {
                    Some(d) => {
                        println!("retrieved: {}", d);
                        log_if_error!(writer.write(format!("{}\n", d).as_bytes()));
                        log_if_error!(writer.flush());
                    }
                    None => {
                        println!("Nothing left in the storage");
                    }
                }
            }
            QUIT => break,
            _ => {}
        };
    }
}

fn get_command(command: String) -> Command {
    use self::Command::*;
    match &command.to_lowercase().trim() as &str {
        c if c.starts_with("publish ") => {
            PUBLISH(command["publish ".len()..].split(",").map(|s| s.trim().into()).collect())
        }
        "retrieve" => RETRIEVE,
        "" => QUIT,
        _ => NONE
    }
}