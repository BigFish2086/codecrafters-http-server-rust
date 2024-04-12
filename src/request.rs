use std::collections::HashMap;
use std::error::Error;
use std::io::{BufReader, BufRead, Read};
use std::net::TcpStream;

#[derive(Debug)]
pub enum Method {
    GET,
    POST
}

impl From<&str> for Method {
    fn from(value: &str) -> Self {
        use crate::Method::*;
        match value {
            ref value if value.to_lowercase() == "get" => GET,
            ref value if value.to_lowercase() == "post" => POST,
            _ => todo!()
        }
    }
}

impl From<String> for Method {
    fn from(value: String) -> Self {
        use crate::Method::*;
        match value {
            ref value if value.to_lowercase() == "get" => GET,
            ref value if value.to_lowercase() == "post" => POST,
            _ => todo!()
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

impl Request {
    pub fn from_stream(mut stream: &mut TcpStream) -> Result <Self, Box<dyn Error>> {
        let mut reader = BufReader::new(&mut stream);
        let mut start_line = String::new();
        if let Err(_) = reader.read_line(&mut start_line) {
            return Err("ERROR: Invalid start line".into());
        }
        let start_line: Vec<String> = start_line
            .split(" ")
            .into_iter()
            .map(|str| str.to_string())
            .collect();
        if start_line.len() != 3 {
            return Err("ERROR: Invalid start line".into());
        }
        let mut body = None;
        let mut content_length = 0;
        let mut headers: HashMap<String, String> = HashMap::new();
        loop {
            let mut line = String::new();
            if let Err(_) = reader.read_line(&mut line) {
                return Err("ERROR: Invalid request header".into());
            }
            if line == "\r\n" {
                break;
            }
            if let Some((header, value)) = line.split_once(":") {
                headers.insert(header.trim().to_lowercase().to_string(), value.trim().to_string());
                if header.trim().to_lowercase() == "content-length" {
                    match value.trim().parse::<usize>() {
                        Ok(value) => content_length = value,
                        Err(_) => return Err("ERROR: Invalid Content-Length header".into()),
                    };
                }
            }
        }
        if content_length > 0 {
            let mut body_content = vec![0; content_length];
            reader.read_exact(&mut body_content).unwrap();
            body = Some(String::from_utf8_lossy(&body_content).to_string());
        }
        Ok(Self {
            method: start_line[0].clone().into(),
            path: start_line[1].clone(),
            version: start_line[2].clone(),
            headers: (!headers.is_empty()).then_some(headers),
            body: body,
        })
    }
}
