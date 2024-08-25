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
    let buf_reader = BufReader::new(stream);
    if let Some(line) = buf_reader.lines().next() {
        match line {
            Ok(s) => {
                if s == "GET / HTTP/1.1" {
                    Ok("HTTP/1.1 200 OK\r\n\r\n".to_string())
                } else {
                    Ok("HTTP/1.1 404 NOT_FOUND\r\n\r\n".to_string())
                }
            }
            Err(e) => Err(e),
        }
    } else {
        Err(Error::other("empty stream"))
    }
}
