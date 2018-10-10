use std::net::{TcpListener, TcpStream};
use std::io;
use std::io::*;

#[derive(Debug)]
enum Command {
    PUBLISH(Vec<String>),
    RETRIEVE,
    NONE
}
//echo "PUBLISH info one, info 2, Grüße, Jürgen ❤" | nc 127.0.0.1 8080
//echo "RETRIEVE" | nc localhost 8080

macro_rules! log_if_error {
    ($e:expr) => {
        if let Err(e) = $e {
            eprint!("There is an error happened: {}", e)
        }
    }
}

fn handle_client(stream: TcpStream, data: &mut Vec<String>) {
    use Command::*;
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let mut buf = String::new();
    log_if_error!(reader.read_line(&mut buf));

    let command = get_command(buf);
    println!("Received command: {:?}", command);
    match command {
        PUBLISH(strings) => {
            println!("publish: {:?}", strings);
            strings.iter().for_each(|s| data.push(s.clone()))
        },
        RETRIEVE => {
            match data.pop() {
                Some(d) => {
                    println!("retrieved: {}", d);
                    log_if_error!(writer.write(format!("{}\n", d).as_bytes()));
                },
                None => {
                    println!("Nothing left in the storage");
                }
            }
        },
        _ => {}
    };
}

fn get_command(command: String) -> Command {
    use Command::*;
    match &command.to_lowercase().trim() as &str {
        c if c.starts_with("publish ") => {
            return PUBLISH(command["publish ".len()..].split(",").map(|s| s.trim().into()).collect())
        },
        "retrieve" => {
            return RETRIEVE
        },
        _ => return NONE
    }
}

fn main() -> io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let mut data: Vec<String> = Vec::new();
    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(stream?, &mut data);
    }
    Ok(())
}

