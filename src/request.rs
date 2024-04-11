use std::collections::HashMap;
use std::error::Error;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
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
            method: method.to_string(),
            path: path.to_string(),
            version: version.to_string(),
            headers: (!headers.is_empty()).then_some(headers),
            body: None,
        })
    }
}
