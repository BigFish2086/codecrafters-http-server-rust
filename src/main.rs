mod request;
mod response;
mod status;

use std::env;
use std::fs;
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
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

fn get_file(req: &Request, directory: &str) -> Response {
    let files_len = "/files/".len();
    let filepath = fs::canonicalize(directory)
        .unwrap()
        .join(&req.path[files_len..]);

    let mut have_read = false;
    let mut file_body = String::new();

    if filepath.exists() && filepath.starts_with(directory) {
        if let Ok(file_input) = fs::File::open(filepath) {
            let mut file_buffered = BufReader::new(file_input);
            if file_buffered.read_to_string(&mut file_body).is_ok() {
                have_read = true;
            }
        }
    }

    if have_read {
        Response {
            status_code: StatusCode::OK,
            headers: Some(vec![(
                "Content-Type".to_string(),
                "application/octet-stream".to_string(),
            )]),
            body: Some(file_body),
        }
    } else {
        Response::new(StatusCode::NOT_FOUND, None, None)
    }
}

fn handle_stream(mut stream: TcpStream, directory: Option<&str>) {
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
            let resp = match req.path {
                ref path if path == "/" => Response::new(StatusCode::OK, None, None),
                ref path if path.starts_with("/echo/") => echo_msg(&req),
                ref path if path.starts_with("/user-agent") => echo_header(&req, "User-Agent"),
                ref path if path.starts_with("/files/") && directory.is_some() => {
                    get_file(&req, directory.unwrap())
                }
                _ => Response::new(StatusCode::NOT_FOUND, None, None),
            };
            if let Err(err) = stream.write_all(&resp.as_bytes()) {
                eprintln!("ERROR: {}", err);
            }
        }
        Err(err) => {
            eprintln!("ERROR: {}", err);
        }
    };
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let directory = match args.len() {
        2 if args[0] == "--directory".to_string() => Some(args[1].clone()),
        _ => None,
    };
    let directory = Arc::new(directory);
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        let directory_clone = Arc::clone(&directory);
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_stream(stream, directory_clone.as_deref()));
            }
            Err(err) => eprintln!("ERROR: {}", err),
        }
    }
}
