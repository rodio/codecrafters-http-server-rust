use std::{
    io::{BufRead, BufReader, Error, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for incoming in listener.incoming() {
        match incoming {
            Ok(mut stream) => {
                match parse_request(&mut stream) {
                    Ok(response) => match stream.write_all(response.as_bytes()) {
                        Ok(_) => (),
                        Err(e) => println!("error while writing response: {}", e),
                    },
                    Err(e) => println!("error while parsing request: {}", e),
                };
            }
            Err(e) => println!("error while creating stream: {}", e),
        };
    }
}

fn parse_request(stream: &mut TcpStream) -> Result<String, Error> {
    let first_line = BufReader::new(stream)
        .lines()
        .next()
        .ok_or(Error::other("empty stream"))?
        .unwrap();

    let mut parts = first_line.split_whitespace();

    let _verb = parts.next().ok_or(Error::other("no HTTP verb"))?;
    let endpoint = parts.next().ok_or(Error::other("no endpoint"))?;

    let ok = String::from("HTTP/1.1 200 OK");
    let crlf = "\r\n";
    let not_found = format!("HTTP/1.1 404 Not Found{crlf}{crlf}");

    match endpoint {
        "/" => Ok(format!("{ok}{crlf}{crlf}")),

        s if s.starts_with("/echo/") => {
            let mut parts = s.split("/echo/");
            parts.next(); // skipping the empty

            if let Some(pong) = parts.next() {
                if pong.is_empty() || pong.contains('/') {
                    Ok(not_found)
                } else {
                    let pong_len = pong.len();
                    Ok(format!("{ok}{crlf}Content-Type: text/plain{crlf}Content-Length: {pong_len}{crlf}{crlf}{pong}"))
                }
            } else {
                Ok(not_found)
            }
        }

        _ => Ok(not_found),
    }
}
