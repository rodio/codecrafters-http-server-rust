mod threadpool;

use std::{
    io::{BufRead, BufReader, Error, Write},
    net::{TcpListener, TcpStream},
};

use threadpool::ThreadPool;

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let mut thread_pool = ThreadPool::new();

    for incoming in listener.incoming() {
        thread_pool.execute(|| match incoming {
            Ok(stream) => {
                match process_request(stream) {
                    Ok(_) => (),
                    Err(e) => println!("error while processing: {}", e),
                };
            }
            Err(e) => println!("error while creating stream: {}", e),
        });
    }
}

fn process_request(mut stream: TcpStream) -> Result<(), Error> {
    let mut lines = BufReader::new(&stream).lines();
    let request_line = lines.next().ok_or(Error::other("empty stream"))??;

    let mut parts = request_line.split_whitespace();

    let _verb = parts.next().ok_or(Error::other("no HTTP verb"))?;
    let endpoint = parts.next().ok_or(Error::other("no endpoint"))?;

    let ok = "HTTP/1.1 200 OK";
    let crlf = "\r\n";
    let not_found = "HTTP/1.1 404 Not Found".to_owned() + crlf + crlf;

    let mut response: String = String::new();

    match endpoint {
        "/" => response = format!("{ok}{crlf}{crlf}"),

        s if s.starts_with("/echo/") => {
            let mut parts = s.split("/echo/");
            parts.next(); // skipping the empty

            if let Some(pong) = parts.next() {
                if pong.is_empty() || pong.contains('/') {
                    response = not_found.to_owned();
                } else {
                    let pong_len = pong.len();
                    response = format!("{ok}{crlf}Content-Type: text/plain{crlf}Content-Length: {pong_len}{crlf}{crlf}{pong}");
                }
            } else {
                response = not_found.to_owned();
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
                    response = format!("{ok}{crlf}Content-Type: text/plain{crlf}Content-Length: {agent_len}{crlf}{crlf}{agent}");
                }
            }
            if response.is_empty() {
                response = not_found.to_owned();
            }
        }

        _ => response = not_found.to_owned(),
    };

    stream.write_all(response.as_bytes())
}
