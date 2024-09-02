use std::{
    collections::HashMap,
    io::{Error, Read},
    net::TcpStream,
};

#[derive(Debug)]
pub(crate) struct Request {
    pub endpoint: String,
    pub verb: String,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
}

impl Request {
    pub fn from_stream(mut stream: &TcpStream) -> Result<Self, Error> {
        let mut buf = [0_u8; 1024];
        let bytes_read = stream.read(&mut buf).unwrap();

        let req_str = match String::from_utf8(buf[..bytes_read].to_vec()) {
            Ok(s) => s,
            Err(e) => return Err(Error::other(format!("non-utf8 request: {}", e))),
        };

        let mut parts = req_str.split("\r\n\r\n");

        let req = parts.next().ok_or(Error::other("bad request format"))?;

        let mut body = None;
        if let Some(b) = parts.next() {
            if !b.is_empty() {
                body = Some(b.to_owned());
            }
        }

        let mut req_lines = req.lines();
        let verb_line = req_lines.next().ok_or(Error::other("empty request"))?;
        let mut verb_line_parts = verb_line.split_whitespace();

        let verb = verb_line_parts
            .next()
            .ok_or(Error::other("no HTTP verb"))?
            .to_owned();
        let endpoint = verb_line_parts
            .next()
            .ok_or(Error::other("no endpoint"))?
            .to_owned();

        let mut headers = HashMap::new();
        for line in req_lines {
            if !line.starts_with("User-Agent: ") {
                continue;
            }
            let mut parts = line.split("User-Agent: ");
            parts.next(); // skip the empty
            let agent = parts.next().unwrap().trim();
            headers.insert("User-Agent".to_owned(), agent.to_owned());
            parts.next();
        }

        Ok(Self {
            endpoint,
            verb,
            body,
            headers,
        })
    }
}
