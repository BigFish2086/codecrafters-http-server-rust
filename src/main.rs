use std::io::{Read, Write};
use std::net::TcpListener;
use std::fmt;
use std::error::Error;

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
            None => text.push_str("\r\n")
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
                            Some(path) if path == "/" => {
                                stream.write_all(&Response::new(StatusCode::OK, None, None).as_bytes())
                            }
                            Some(path) if path.starts_with("/echo/") => {
                                let echo_len = "/echo/".len();
                                let body = &path[echo_len..];
                                let resp = Response {
                                    status_code: StatusCode::OK,
                                    headers: Some(vec!{ ("Content-Type".to_string(), "text/plain".to_string()) }),
                                    body: Some(body.to_string())
                                };
                                stream.write_all(&resp.as_bytes())
                            }
                            _ => {
                                stream.write_all(&Response::new(StatusCode::NOT_FOUND, None, None).as_bytes())
                            }
                        } {
                            print!("ERROR: {}", err);
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
