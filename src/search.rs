use std::{
    io::{
        Read,
        Write
    },
    net::{
        SocketAddr,
        TcpStream,
        ToSocketAddrs,
    },
    string::FromUtf8Error,
};

use crate::{
    error::{
        NoneError,
        error,
    },
    tcp::{
        BUFFER_SIZE,
        RequestBase,
    },
};

pub async fn search(query: &str, api_key: &str) -> Vec<u8> {
    println!("Here");

    let address: SocketAddr = match ("serpapi.com:80").to_socket_addrs() {
        Ok(addresses) => {
            let addresses_collected: Vec<SocketAddr> = addresses.collect();

            addresses_collected[0] // TODO Find a better way of selecting a socket address
        },
        Err(_) => {
            error::<NoneError>("DNS resolution failed", &NoneError);

            return SearchError::service_unavaliable();
        },
    };

    println!("{:?}", address);

    let mut stream: TcpStream = match TcpStream::connect(address) {
        Ok(s) => s,
        Err(_) => {
            error::<NoneError>("Could not connect to search API", &NoneError);

            return SearchError::service_unavaliable();
        },
    };

    let mut request: Vec<u8> = vec![];
    request = (format!("GET /search.json?engine=duckduckgo_light&q={}&api_key={}", query, api_key)).as_bytes().to_vec();
    request.add_header("User-Agent", "Astra");
    request.add_header("Accept", "application/json");

    // TODO verify all written
    stream.write(&request);

    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

    println!("{:?}", stream.read(&mut buffer));

    let stream_string: String = match String::from_utf8(buffer.to_vec()) {
        Ok(s) => s,
        Err(e) => {
            error::<FromUtf8Error>("Could not convert TCP bytes to UTF-8 string", &e);
            "".to_string()
        },
    };

    let mut stream_string_vec: Vec<&str> = stream_string.split("\r\n").collect();

    println!("{:?}", stream_string_vec);

    todo!();
}

async fn do_search(query: String, api_key: &str) -> String {
    println!("Querying for: {}", query);

    let res: String = format!("https://serpapi.com/search.json?engine=duckduckgo_light&q={}&api_key={}", query, api_key);

    res
}

struct SearchError {}

trait Errors {
    fn service_unavaliable() -> Vec<u8>;
}

impl Errors for SearchError {
    fn service_unavaliable() -> Vec<u8> {
        let mut response: Vec<u8> = vec![];

        response = ("HTTP/1.1 503 Service Unavaliable\r\n").as_bytes().to_vec();

        response.add_header("Server", "Astra");
        response.add_header("Content-Type", "application/octet-stream");

        response.add_header("Retry-After", "60");

        response.add_header("Content-Length", "0");

        response
    }
}