use std::collections::HashMap;
use std::error::Error;

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
    pub fn from_utf8(buffer: &[u8]) -> Result<Self, Box<dyn Error>> {
        let content = std::str::from_utf8(&buffer).unwrap();
        if let Some((req, body)) = content.split_once("\r\n\r\n") {
            let req: Vec<_> = req.split("\r\n").collect();

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
                method: method.into(),
                path: path.to_string(),
                version: version.to_string(),
                headers: (!headers.is_empty()).then_some(headers),
                body: (!body.is_empty()).then_some(body.to_string()),
            })
        } else {
            Err("ERROR: Invalid Request".into())
        }
    }
}
