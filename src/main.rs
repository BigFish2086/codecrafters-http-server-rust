use std::net::TcpListener;
use std::io::{Read, Write};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("INFO: accepted new connection");
                let mut buffer = [0u8; 1024];
                match stream.read(&mut buffer) {
                    Ok(buffer_len) => {
                        println!("INFO: Read {} bytes\n{}", buffer_len, std::str::from_utf8(&buffer[..buffer_len]).unwrap());
                        if let Err(err) = stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n") {
                            print!("Error: {}", err);
                        }
                    }
                    Err(err) => { println!("ERROR: {}", err); }
                };
            }
            Err(err) => {
                println!("ERROR: {}", err);
            }
        }
    }
}
