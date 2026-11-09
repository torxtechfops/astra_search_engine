use std::{
    io::{
        BufRead,
        BufReader,
        Write,
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
        api_error,
        error,
    },
    search::{
        search_errors::{
            HTTPError,
            NoResultsError
        },
        search_result::{
            SearchResult,
            Json,
        },
    },
    tcp::tcp_request::RequestBase,
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
    request = (format!("GET /search.json?engine=duckduckgo_light&q={}&api_key={}&json_restrictor=organic_results HTTP/1.1\r\n", query, api_key))
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
    while !recieved.ends_with("\n}".as_bytes()) {
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

    // Handle this error potential
    let stream_string: String = match String::from_utf8(recieved.clone()) {
        Ok(s) => s,
        Err(e) => {
            error::<FromUtf8Error>("Could not convert response to UTF-8", &e);

            return SearchError::unknown_error("Couldn't process gateway response");
        },
    };
    
    ////println!("{}", stream_string);

    let stream_string_vec: Vec<&str> = stream_string.split("\r\n").collect();

    // [0]HTTPS/1.1 [1]200 [3]OK
    let status_number: &str = stream_string_vec[0].split(" ").collect::<Vec<&str>>()[1];
    let mut bad_gateway: bool = true;
    match status_number {
        // See https://serpapi.com/api-status-and-error-codes
        "200" => {
            bad_gateway = false;
        },
        "400" => {
            api_error("Bad request to API server", status_number);
        },
        "401" => {            
            api_error("API key couldn't be found by server", status_number);
        },
        "403" => {
            api_error("API key expired or invalid", status_number);
        },
        "404" => {
            api_error("API resource could not be found", status_number);
        },
        "410" => {
            api_error("API resource no longer exists", status_number);
        },
        "429" => {
            api_error("API rate limit exceeded", status_number);
        },
        "500" => {
            api_error("Unknown API error", status_number);
        },
        "503" => {
            api_error("API service unavaliable", status_number);
        },
        _ => {
            api_error("Unknown API error", status_number);
            error::<NoneError>("Unexpected response from API server", &NoneError);
        },
    };

    if bad_gateway {
        return SearchError::bad_gateway(status_number)
    };

    let results: Vec<SearchResult> = match get_results(stream_string) {
        Ok(r) => r,
        Err(e) => {
            error::<NoResultsError>("No results obtained from API response", &e);

            return SearchError::bad_gateway("500")
        },
    };

    let mut body: Vec<u8> = b"[\n".to_vec();
    let mut index: usize = 1;
    let len: usize = results.len();
    for result in results {
        body.append(&mut result.into_json().as_bytes().to_vec());

        if index < len {
            body.append(&mut b",".to_vec());
        };
        index += 1;
    };

    body.append(&mut b"\n]".to_vec());

    let mut response: Vec<u8> = ("HTTP/1.1 200 Ok \r\n").as_bytes().to_vec();
    response.add_header("Server", "Astra");
    response.add_header("Content-Type", "application/json");
    response.add_header("Cache-Control", "max-age=600");

    response.add_header("Content-Length", &body.len().to_string());

    response.add_body(body);
    
    response
}

fn get_results(response: String) -> Result<Vec<SearchResult>, NoResultsError> {
    let response_split: Option<(&str, &str)> = response.split_once("\r\n\r\n");

    let body_vec: Vec<&str> = match response_split {
        Some((_, body)) => {
            body.split("\n").collect::<Vec<&str>>()
        },
        None => {
            error::<NoResultsError>("Error getting results", &NoResultsError);

            return Err(NoResultsError)
        },
    };

    let mut results: Vec<SearchResult> = vec![];

    let mut body_iter: std::slice::Iter<'_, &str> = body_vec.iter();
    loop {
        let line = match body_iter.next() {
            Some(l) => l,
            None => {
                break;
            },
        };

        ////println!("{}", line);

        if line.contains(&str_to_json_field("title")) {
            results.push(SearchResult::default());

            let line_vec: Vec<&str> = line.split(&str_to_json_field("title")).collect::<Vec<&str>>();

            if line_vec.len() == 2 {
                let len: usize = results.len() - 1;
                results[len].title = line_vec[1].to_string();
            };
        } else if line.contains(&str_to_json_field("link")) {
            let line_vec: Vec<&str> = line.split(&str_to_json_field("link")).collect::<Vec<&str>>();

            if line_vec.len() == 2 {
                let len: usize = results.len() - 1;
                results[len].link = line_vec[1].to_string();
            };
        } else if line.contains(&str_to_json_field("snippet")) {
            let line_vec: Vec<&str> = line.split(&str_to_json_field("snippet")).collect::<Vec<&str>>();

            if line_vec.len() == 2 {
                let len: usize = results.len() - 1;
                let snippet: &str = line_vec[1].trim_end_matches(",");
                results[len].snippet = snippet.to_string();
            };
        } else if line.contains("]") {
            break;
        };
    };

    if results.len() == 0 {
        return Err(NoResultsError);
    };

    ////results.display_all();

    return Ok(results)
}

fn str_to_json_field(input: &str) -> String {
    return format!("\"{}\": ", input)
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