use std::{
    fmt,
    io::{
        BufRead, BufReader, Write,
    },
    net::{
        SocketAddr,
        TcpStream,
        ToSocketAddrs,
    },
    string::FromUtf8Error,
};

use native_tls::{
    Protocol,
    TlsConnector,
};

use crate::{
    error::{
        NoneError,
        error,
    },
    tcp::{
        RequestBase,
    },
};

pub async fn search(mut query: &str, api_key: &str) -> Vec<u8> {
    // Remove "/search_query?"
    let query_string: String = query.replacen("/search_query?", "", 1);
    query = &query_string;

    let address: SocketAddr = match ("serpapi.com:443").to_socket_addrs() {
        Ok(addresses) => {
            let addresses_collected: Vec<SocketAddr> = addresses.collect();

            addresses_collected[0] // TODO Find a better way of selecting a socket address
        },
        Err(_) => {
            error::<NoneError>("DNS resolution failed", &NoneError);

            return SearchError::service_unavaliable();
        },
    };

    ////println!("{:?}", address);

    let tls: TlsConnector = match TlsConnector::builder()
        .min_protocol_version(None)
        .max_protocol_version(Some(Protocol::Tlsv13))
        .build() {
        Ok(t) => t,
        Err(_) => {
            error::<NoneError>("Failed to build TLS connector", &NoneError);

            return SearchError::service_unavaliable();
        },
    };

    let stream: TcpStream = match TcpStream::connect(address) {
        Ok(s) => s,
        Err(_) => {
            error::<NoneError>("Unable to create TCP stream", &NoneError);

            return SearchError::service_unavaliable();
        },
    };
    // match stream.set_read_timeout(Some(Duration::from_millis(60000))) {
    //     Ok(_) => {},
    //     Err(_) => {
    //         error::<NoneError>("Unable to set read timeout for TCP Stream", &NoneError);
    //     },
    // };

    let mut tls_stream: native_tls::TlsStream<TcpStream> = match tls.connect("serpapi.com", stream) {
        Ok(t) => t,
        Err(e) => {
            error::<native_tls::HandshakeError<TcpStream>>("Handshake error with API server", &e);

            return SearchError::service_unavaliable();
        },
    };

    let mut request: Vec<u8> = vec![];
    request = (format!("GET /search.json?engine=duckduckgo_light&q={}&api_key={} HTTP/1.1\r\n", query, api_key))
        .as_bytes()
        .to_vec();

    request.add_header("Host", "serpapi.com");
    request.add_header("User-Agent", "Astra/0.1");
    ////request.add_header("Accept", "*/*");
    request.add_header("Connection", "Keep-Alive");
    ////request.add_header("Accept-Encoding", "gzip, deflate, br, zstd");

    request.append(&mut b"\r\n\r\n".to_vec());

    // Verify all written
    match tls_stream.write(&request) {
        Ok(u) => {
            if u != request.len() {
                error::<NoneError>(&format!("Failed to write all bytes to TCP stream {}/{}", u, request.len()), &NoneError);
            }
        },
        Err(e) => {
            error::<std::io::Error>("Failed to write bytes to TCP stream", &e);

            return SearchError::service_unavaliable();
        },
    };

    // Required as TCP data may not all be written and we must verify all content has been recieved
    let mut reader: BufReader<&mut native_tls::TlsStream<TcpStream>> = BufReader::new(&mut tls_stream);

    let mut recieved: Vec<u8> = vec![];

    // Why does the API refuse to end with a new line? who can say...
    while !recieved.ends_with("}\n}".as_bytes()) {
        let to_add: Vec<u8> = match reader.fill_buf() {
            Ok(v) => v.to_vec(),
            Err(e) => {
                error::<std::io::Error>("Couldn't read TCP stream to buffer", &e);

                return SearchError::unknown_error("Couldn't read gateway response");
            },
        };

        reader.consume(to_add.len());

        recieved = [recieved, to_add].concat();

        ////println!("{:?}", recieved);
    };

    let stream_string: String = String::from_utf8(recieved.clone()).unwrap();
    
    println!("{}", stream_string);

    panic!();

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

    fn bad_gateway(status: &str) -> Vec<u8>;

    fn unknown_error(info: &str) -> Vec<u8>;
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

    fn bad_gateway(status: &str) -> Vec<u8> {
        //Bad gateway, down!
        error::<HTTPError>("Bad gateway response encountered", &HTTPError(status.to_string()));

        let mut response: Vec<u8> = vec![];

        response = ("HTTP/1.1 502 Bad Gateway\r\n").as_bytes().to_vec();

        response.add_header("Server", "Astra");
        response.add_header("Content-Type", "text/plain");

        let body: Vec<u8> = status.as_bytes().to_vec();

        response.add_header("Content-Length", &body.len().to_string());

        response.add_body(body);

        response
    }

    fn unknown_error(information: &str) -> Vec<u8> {
        //Bad gateway, down!
        error::<HTTPError>("Error parsing API response", &HTTPError(information.to_string()));

        let mut response: Vec<u8> = vec![];

        response = ("HTTP/1.1 500 Internal Server Error\r\n").as_bytes().to_vec();

        response.add_header("Server", "Astra");
        response.add_header("Content-Type", "text/plain");

        let body: Vec<u8> = information.as_bytes().to_vec();

        response.add_header("Content-Length", &body.len().to_string());

        response.add_body(body);

        response
    }
}

#[derive(Debug, Clone)]
struct HTTPError(String);

impl fmt::Display for HTTPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for HTTPError {}