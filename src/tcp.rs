use std::{io::{self, BufRead, BufReader, Error, Read}, net::{
    Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream,
}, string::FromUtf8Error, time::Duration};

use tokio::runtime::Runtime;

use crate::error::{NoneError, error, fatal_error};

const BUFFER_SIZE: usize = 8192;

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

    println!("{}", stream_string);
}

#[allow(dead_code)]
fn display_vec(vector: Vec<u8>) {
    for index in vector {
        if index != 0 {
            print!("{}, ", index);
        };
    };

    println!();
}
