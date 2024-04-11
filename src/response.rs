use std::fmt;

use crate::status::StatusCode;

#[derive(Debug)]
pub struct Response {
    pub status_code: StatusCode,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<String>,
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
