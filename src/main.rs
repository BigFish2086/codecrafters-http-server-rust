mod request;
mod response;
mod status;

use std::env;
use std::fs;
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use crate::request::{Request, Method};
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
        Some(map) => map.get(header.trim()).unwrap_or(&binding),
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

fn post_file(req: &Request, directory: &str) -> Response {
    let files_len = "/files/".len();
    let filepath = fs::canonicalize(directory)
        .unwrap()
        .join(&req.path[files_len..]);

    let mut have_written = false;
    if let Ok(file_input) = fs::File::create(filepath) {
        let mut file_buffered = BufWriter::new(file_input);
        if req.body.is_some() && file_buffered.write_all(req.body.as_ref().unwrap().as_bytes()).is_ok() {
            have_written = true;
        }
    }

    if have_written {
        Response::new(StatusCode::CREATED, None, None)
    } else {
        Response::new(StatusCode::INTERNAL_SERVER_ERROR, None, None)
    }
}

fn handle_stream(mut stream: TcpStream, directory: Option<&str>) {
    println!("INFO: Incoming Connection {:?}", stream);
    let req = Request::from_stream(&mut stream).unwrap();
    let resp = match (&req.method, &req.path) {
        (Method::GET, path) if *path == "/" => Response::new(StatusCode::OK, None, None),
        (Method::GET, path) if path.starts_with("/echo/") => echo_msg(&req),
        (Method::GET, path) if path.starts_with("/user-agent") => echo_header(&req, "user-agent"),
        (Method::GET, path) if path.starts_with("/files/") && directory.is_some() => {
            get_file(&req, directory.unwrap())
        }
        (Method::POST, path) if path.starts_with("/files/") && directory.is_some() => { 
            post_file(&req, directory.unwrap())
        }
        _ => Response::new(StatusCode::NOT_FOUND, None, None),
    };
    if let Err(err) = stream.write_all(&resp.as_bytes()) {
        eprintln!("ERROR: {}", err);
    }
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
