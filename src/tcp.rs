use std::{
    fs,
    io::{Read, Write}, net::{
        Ipv4Addr,
        SocketAddr,
        SocketAddrV4,
        TcpListener,
        TcpStream,
    }, string::FromUtf8Error,
};

use crate::{
    error::{
        NoneError,
        error,
    },
    tcp_errors::{
        StreamTooLongError,
        UnknownRequestTypeError,
    },
};

pub const BUFFER_SIZE: usize = 8192; // This is the default size for a BufReader as well

pub async fn handle_listener(order: &str, listener: TcpListener) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("({}) | Socket handler thread at {} initialised correctly!", order, match listener.local_addr() {
        Ok(a) => a.to_string(),
        Err(_) => {
            error::<NoneError>("Couldn't get local address for TCP Listener", &NoneError);
            "(!unknown!)".to_string()
        },
    });
    
    loop {
        let (request_stream, request_address): (std::net::TcpStream, std::net::SocketAddr) = listener.accept()?;

        handle_stream(request_stream, request_address).await
    };
}

pub fn default_socket() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080))
}

async fn handle_stream(mut stream: TcpStream, socket: SocketAddr) {
    println!("Handling {}...", socket);

    let mut buf: Vec<u8> = vec![0; BUFFER_SIZE];

    match stream.read(&mut buf) {
        Ok(_) => (),
        Err(_) => {
            error::<NoneError>("Couldn't read TCP Stream to buffer", &NoneError);
        },
    };

    let stream_string: String = match String::from_utf8(buf) {
        Ok(s) => s,
        Err(e) => {
            error::<FromUtf8Error>("Could not convert TCP bytes to UTF-8 string", &e);
            "".to_string()
        },
    };

    let mut stream_string_vec: Vec<&str> = stream_string.split("\r\n").collect();
    if stream_string_vec[stream_string_vec.len() - 1].contains("\0") {
        stream_string_vec.pop();// All the remaining \0 from the buffer
    } else {
        if stream_string_vec[stream_string_vec.len() - 2] != "" {
            error::<StreamTooLongError>("Could not fully read TCP stream", &StreamTooLongError);
        };
    };

    // Get response based on request type
    let mut response: Vec<u8> = vec![];
    if stream_string_vec[0].contains("GET") {
        println!("GET request");

        response = get_request(&stream_string_vec);
    } else if stream_string_vec[0].contains("POST") {
        println!("POST request");

        todo!();
    } else {
        error::<UnknownRequestTypeError>(&("Could not match request type in header ".to_owned() + stream_string_vec[0]), &UnknownRequestTypeError);
    };

    stream.write(&response);

    ////println!("{:?}", stream_string_vec);
}

#[allow(dead_code)]
fn display_vec(vector: &Vec<u8>) {
    for index in vector {
        if *index != 0 {
            print!("{}, ", index);
        };
    };

    println!();
}

fn get_request(request: &Vec<&str>) -> Vec<u8> {
    let header_first: Vec<&str> = request[0].split(" ").collect();
    let resource: &str = header_first[1];

    let page: Vec<u8> = fs::read("pages/404.html").unwrap();

    // TODO get correct page
    // TODO ensure requested resource is safe for transmission (do not sent conf file)
    // TODO get return MIME type

    let mut response: Vec<u8> = vec![];

    response = b"HTTP/1.1 200 OK\r\n".to_vec();

    response.add_header("Server", "Astra");
    response.add_header("Content-Type", "text/html; charset=utf-8");
    response.add_header("Cache-Control", "max-age=600");

    response.add_header("Content-Length", &page.len().to_string());
    response.add_body(page);

    display_vec(&response);

    response
}

trait RequestBase {
    fn add_header(&mut self, header: &str, value: &str) -> Self;

    fn add_body(&mut self, data: Vec<u8>) -> Self;
}

impl RequestBase for Vec<u8> {
    fn add_header(&mut self, header: &str, value: &str) -> Self {
        let mut addition: Vec<u8> = (header.to_string() + ": " + value + "\r\n").as_bytes().to_vec();

        self.append(&mut addition);

        self.to_owned()
    }

    fn add_body(&mut self, data: Vec<u8>) -> Self {
        let mut body: Vec<u8> = [[b"\r\n".to_vec(), data].concat(), b"\r\n".to_vec()].concat();

        self.append(&mut body);
        
        self.to_owned()
    }
}