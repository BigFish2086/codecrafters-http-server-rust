mod request;
mod response;
mod status;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::request::Request;
use crate::response::Response;
use crate::status::StatusCode;

fn echo_msg(req: &Request) -> Response {
    let echo_len = "/echo/".len();
    let resp_body = &req.path[echo_len..];
    Response {
        status_code: StatusCode::OK,
        headers: Some(vec![("Content-Type".to_string(), "text/plain".to_string())]),
        body: Some(resp_body.to_string()),
    }
}

fn echo_header(req: &Request, header: &str) -> Response {
    let binding = String::new();
    let resp_body = match &req.headers {
        Some(map) => map.get(header).unwrap_or(&binding),
        None => "",
    };
    Response {
        status_code: StatusCode::OK,
        headers: Some(vec![("Content-Type".to_string(), "text/plain".to_string())]),
        body: Some(resp_body.trim().to_string()),
    }
}

fn handle_stream(mut stream: TcpStream) {
    println!("INFO: Incoming Connection {:?}", stream);
    let mut buffer = [0u8; 1024];
    match stream.read(&mut buffer) {
        Ok(buffer_len) => {
            let req = match Request::from_utf8(&buffer[..buffer_len]) {
                Ok(req) => req,
                Err(err) => {
                    eprintln!("ERROR: {}", err);
                    return;
                }
            };
            if let Err(err) = match req.path {
                ref path if path == "/" => {
                    stream.write_all(&Response::new(StatusCode::OK, None, None).as_bytes())
                }
                ref path if path.starts_with("/echo/") => {
                    stream.write_all(&echo_msg(&req).as_bytes())
                }
                ref path if path.starts_with("/user-agent") => {
                    stream.write_all(&echo_header(&req, "User-Agent").as_bytes())
                }
                _ => stream.write_all(&Response::new(StatusCode::NOT_FOUND, None, None).as_bytes()),
            } {
                eprintln!("ERROR: {}", err);
            }
        }
        Err(err) => {
            eprintln!("ERROR: {}", err);
        }
    };
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_stream(stream));
            }
            Err(err) => eprintln!("ERROR: {}", err),
        }
    }
}
