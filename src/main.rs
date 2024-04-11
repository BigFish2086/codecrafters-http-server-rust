use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatusCode(u16);

pub struct InvalidStatusCode {
    _priv: (),
}

impl StatusCode {
    #[inline]
    pub fn from_u16(num: u16) -> Result<StatusCode, InvalidStatusCode> {
        if !(100..1000).contains(&num) {
            Err(InvalidStatusCode::new())
        } else {
            Ok(StatusCode(num))
        }
    }

    #[inline]
    pub fn as_u16(&self) -> u16 {
        self.0
    }

    pub fn reason(&self) -> Option<&'static str> {
        reason(self.0)
    }
}

impl fmt::Debug for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.0,
            self.reason().unwrap_or("<unknown status code>")
        )
    }
}

impl InvalidStatusCode {
    fn new() -> InvalidStatusCode {
        InvalidStatusCode { _priv: () }
    }
}

impl fmt::Debug for InvalidStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InvalidStatusCode").finish()
    }
}

impl fmt::Display for InvalidStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid status code")
    }
}

impl Error for InvalidStatusCode {}

macro_rules! status_codes {
    ( $( ($num:expr, $konst:ident, $phrase:expr);)+) => {
        impl StatusCode {
            $( pub const $konst: StatusCode = StatusCode($num); )+
        }
        fn reason(num: u16) -> Option<&'static str> {
            match num {
                $( $num => Some($phrase),)+
                _ => None
            }
        }
    }
}

status_codes! {
    (200, OK, "OK");
    (404, NOT_FOUND, "Not Found");
}

#[derive(Debug)]
struct Response {
    status_code: StatusCode,
    headers: Option<Vec<(String, String)>>,
    body: Option<String>,
}

impl Response {
    pub fn new(
        status_code: StatusCode,
        headers: Option<Vec<(String, String)>>,
        body: Option<String>,
    ) -> Self {
        Self {
            status_code,
            headers,
            body,
        }
    }

    pub fn as_string(&self) -> String {
        let mut text = String::new();
        text.push_str(format!("HTTP/1.1 {}\r\n", &self.status_code).as_str());
        if let Some(headers) = &self.headers {
            for (header, value) in headers {
                text.push_str(format!("{}: {}\r\n", header, value).as_str());
            }
        }
        match &self.body {
            Some(body) => {
                let body_len = body.len();
                text.push_str(format!("Content-Length: {}\r\n\r\n{}", body_len, body).as_str());
            }
            None => text.push_str("\r\n"),
        }
        return text;
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        return self.as_string().into();
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    version: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>
}

impl Request {
    pub fn from_utf8(buffer: &[u8]) -> Result<Self, Box<dyn Error>> {
        let content = std::str::from_utf8(&buffer).unwrap();
        let content: Vec<_> = content.split("\r\n\r\n").collect();
        let req: Vec<_> = content[0].split("\r\n").collect();

        let status_line: Vec<_> = req[0].split(" ").collect();
        if status_line.len() != 3 {
            return Err("ERROR: Invalid status line".into());
        }
        let [method, path, version] = status_line[..3].try_into().unwrap();

        let mut headers: HashMap<String, String> = HashMap::new();
        for line in &req[1..] {
            if let Some((header, value)) = line.split_once(":") {
                headers.insert(header.to_string(), value.to_string());
            }
        }

        Ok(Self {
            method:  method.to_string(),
            path:    path.to_string(),
            version: version.to_string(),
            headers: (!headers.is_empty()).then_some(headers),
            body:    None,
        })
    }
}

fn echo_msg(req: &Request) -> Response {
    let echo_len = "/echo/".len();
    let resp_body = &req.path[echo_len..];
    Response {
        status_code: StatusCode::OK,
        headers: Some(vec![("Content-Type".to_string(), "text/plain".to_string(),)]),
        body: Some(resp_body.to_string()),
    }
}

fn echo_header(req: &Request, header: &str) -> Response {
    let binding = String::new();
    let resp_body = match &req.headers {
        Some(map) => map.get(header).unwrap_or(&binding),
        None => ""
    };
    Response {
        status_code: StatusCode::OK,
        headers: Some(vec![("Content-Type".to_string(), "text/plain".to_string(),)]),
        body: Some(resp_body.trim().to_string()),
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("INFO: accepted new connection");
                let mut buffer = [0u8; 1024];
                match stream.read(&mut buffer) {
                    Ok(buffer_len) => {
                        let req = Request::from_utf8(&buffer[..buffer_len]).unwrap();
                        if let Err(err) = match req.path {
                            path if path == "/" => stream.write_all(&Response::new(StatusCode::OK, None, None).as_bytes()),
                            ref path if path.starts_with("/echo/") => stream.write_all(&echo_msg(&req).as_bytes()),
                            ref path if path.starts_with("/user_agent") => stream.write_all(&echo_header(&req, "User-Agent").as_bytes()),
                            _ => stream.write_all(&Response::new(StatusCode::NOT_FOUND, None, None).as_bytes()),
                        } {
                            println!("ERROR: {}", err);
                        }
                    }
                    Err(err) => {
                        println!("ERROR: {}", err);
                    }
                };
            }
            Err(err) => {
                println!("ERROR: {}", err);
            }
        }
    }
}
