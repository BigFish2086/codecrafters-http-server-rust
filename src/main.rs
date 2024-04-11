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
                        let content = std::str::from_utf8(&buffer[..buffer_len]).unwrap();
                        if let Err(err) = match content.split(" ").into_iter().skip(1).next() {
                            Some(path) if path == "/" => stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n"),
                            Some(path) if path.starts_with("/echo/") => {
                                let echo_len = "/echo/".len();
                                let body = &path[echo_len..];
                                let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n", body.len(), body);
                                println!("{}", response);
                                stream.write_all(&response.as_bytes())
                            }
                            _ => stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                        } {
                            print!("ERROR: {}", err);
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
