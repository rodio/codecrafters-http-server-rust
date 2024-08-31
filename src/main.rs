mod threadpool;

use std::{
    env,
    io::{BufRead, BufReader, Error, Read, Write},
    net::{TcpListener, TcpStream},
};

use threadpool::ThreadPool;

const OK: &[u8] = b"HTTP/1.1 200 OK";
const CRLF: &[u8] = b"\r\n";
const NOT_FOUND: &[u8] = b"HTTP/1.1 404 Not Found\r\n\r\n";

fn main() {
    println!("Logs from your program will appear here!");
    let mut directory: Option<String> = None;
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 && args.get(1).unwrap() == "--directory" {
        directory = Some(args[2].to_owned());
    }

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let mut thread_pool = ThreadPool::new();

    for incoming in listener.incoming() {
        let dir = directory.clone();
        thread_pool.execute(|| match incoming {
            Ok(stream) => {
                match process_request(stream, dir) {
                    Ok(_) => (),
                    Err(e) => println!("error while processing: {}", e),
                };
            }
            Err(e) => println!("error while creating stream: {}", e),
        });
    }
}

fn process_request(mut stream: TcpStream, directory: Option<String>) -> Result<(), Error> {
    let mut lines = BufReader::new(&stream).lines();
    let request_line = lines.next().ok_or(Error::other("empty stream"))??;

    let mut parts = request_line.split_whitespace();

    let _verb = parts.next().ok_or(Error::other("no HTTP verb"))?;
    let endpoint = parts.next().ok_or(Error::other("no endpoint"))?;

    match endpoint {
        "/" => return stream.write_all([OK, CRLF, CRLF].concat().as_slice()),

        s if s.starts_with("/echo/") => {
            let mut parts = s.split("/echo/");
            parts.next(); // skipping the empty
            if let Some(pong) = parts.next() {
                return serve_echo(pong, stream);
            }
        }

        s if s.starts_with("/files/") => {
            let mut parts = s.split("/files/");
            parts.next(); // skipping the empty
            if let Some(filename) = parts.next() {
                return serve_file(directory, filename, stream);
            }
            return stream.write_all(NOT_FOUND);
        }

        "/user-agent" => {
            for line in lines
                .map(|line| line.unwrap())
                .take_while(|line| !line.is_empty())
            {
                if !line.starts_with("User-Agent: ") {
                    continue;
                }
                let mut parts = line.split("User-Agent: ");
                parts.next();
                let agent = parts.next().unwrap().trim().as_bytes();
                let agent_len = agent.len().to_string();
                return stream.write_all(
                    [
                        OK,
                        CRLF,
                        b"Content-Type: text/plain",
                        CRLF,
                        b"Content-Length: ",
                        agent_len.as_bytes(),
                        CRLF,
                        CRLF,
                        agent,
                    ]
                    .concat()
                    .as_slice(),
                );
            }

            // Header not found
            return stream.write_all(NOT_FOUND);
        }

        _ => return stream.write_all(NOT_FOUND),
    };

    stream.write_all(NOT_FOUND)
}

fn serve_file(
    directory: Option<String>,
    filename: &str,
    mut stream: TcpStream,
) -> Result<(), Error> {
    if filename.is_empty() || filename.contains('/') || filename.contains("..") {
        return stream.write_all(NOT_FOUND);
    }

    let dir = directory.unwrap_or(String::from("."));

    let mut file = match std::fs::File::open(format!("{dir}/{filename}")) {
        Ok(file) => file,
        Err(e) => {
            stream.write_all(NOT_FOUND);
            return Err(e);
        }
    };

    let mut content: Vec<u8> = vec![];
    match file.read_to_end(&mut content) {
        Ok(bytes_read) => {
            return stream.write_all(
                [
                    OK,
                    CRLF,
                    b"Content-Type: application/octet-stream",
                    CRLF,
                    b"Content-Length: ",
                    bytes_read.to_string().as_bytes(),
                    CRLF,
                    CRLF,
                    content.as_slice(),
                ]
                .concat()
                .as_slice(),
            );
        }
        Err(e) => {
            stream.write_all(NOT_FOUND);
            Err(e)
        }
    }
}

fn serve_echo(echo: &str, mut stream: TcpStream) -> Result<(), Error> {
    if echo.is_empty() || echo.contains('/') {
        return stream.write_all(NOT_FOUND);
    }

    let pong_len = echo.len().to_string();
    return stream.write_all(
        [
            OK,
            CRLF,
            b"Content-Type: text/plain",
            CRLF,
            b"Content-Length: ",
            pong_len.as_bytes(),
            CRLF,
            CRLF,
            echo.as_bytes(),
        ]
        .concat()
        .as_slice(),
    );
}
