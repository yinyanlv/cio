extern crate cio;

use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use cio::ThreadPool;

fn main() {

    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(5) {

        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("server shutting down");
}

fn handle_connection(mut stream: TcpStream) {

    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "resources/index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "resources/404.html")
    };

    let mut file = File::open(filename).unwrap();
    let mut body = String::new();

    file.read_to_string(&mut body).unwrap();

    let res = format!("{}{}", status_line, body);

    stream.write(res.as_bytes()).unwrap();
    stream.flush().unwrap();
}
