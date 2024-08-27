use std::{
    io::{BufRead, BufReader, Error, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let mut handles = vec![];

    for incoming in listener.incoming() {
        let handle = thread::spawn(|| match incoming {
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
        });
        handles.push(handle);
    }

    for h in handles {
        h.join();
    }
}

fn parse_request(stream: &mut TcpStream) -> Result<String, Error> {
    let mut lines = BufReader::new(stream).lines();
    let request_line = lines.next().ok_or(Error::other("empty stream"))?.unwrap();

    let mut parts = request_line.split_whitespace();

    let _verb = parts.next().ok_or(Error::other("no HTTP verb"))?;
    let endpoint = parts.next().ok_or(Error::other("no endpoint"))?;

    let ok = "HTTP/1.1 200 OK";
    let crlf = "\r\n";
    let not_found = "HTTP/1.1 404 Not Found".to_owned() + crlf + crlf;

    let mut result: String = String::new();

    match endpoint {
        "/" => result = format!("{ok}{crlf}{crlf}"),

        s if s.starts_with("/echo/") => {
            let mut parts = s.split("/echo/");
            parts.next(); // skipping the empty

            if let Some(pong) = parts.next() {
                if pong.is_empty() || pong.contains('/') {
                    result = not_found.to_owned();
                } else {
                    let pong_len = pong.len();
                    result = format!("{ok}{crlf}Content-Type: text/plain{crlf}Content-Length: {pong_len}{crlf}{crlf}{pong}");
                }
            } else {
                result = not_found.to_owned();
            }
        }

        "/user-agent" => {
            for line in lines
                .map(|line| line.unwrap())
                .take_while(|line| !line.is_empty())
            {
                if line.starts_with("User-Agent: ") {
                    let mut parts = line.split("User-Agent: ");
                    parts.next();
                    let agent = parts.next().unwrap().trim();
                    let agent_len = agent.len();
                    result = format!("{ok}{crlf}Content-Type: text/plain{crlf}Content-Length: {agent_len}{crlf}{crlf}{agent}");
                }
            }
            if result.is_empty() {
                result = not_found.to_owned();
            }
        }

        _ => result = not_found.to_owned(),
    };

    Ok(result)
}
