use std::{
    fs,
    io::{
        Read,
        Write
    },
    net::{
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
        ResponseNotFullyWrittenError,
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
    ////println!("Handling {}...", socket);

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
        ////println!("GET request");

        response = get_request(&stream_string_vec);
    } else if stream_string_vec[0].contains("POST") {
        ////println!("POST request");

        todo!();
    } else {
        error::<UnknownRequestTypeError>(&("Could not match request type in header ".to_owned() + stream_string_vec[0]), &UnknownRequestTypeError);
    };

    match stream.write(&response) {
        Ok(u) => {
            if u != response.len() {
                error::<ResponseNotFullyWrittenError>("Couldn't write all response bytes to stream", &ResponseNotFullyWrittenError(u, response.len()));              
            };
        },
        Err(_) => {
            error::<NoneError>("Couldn't write to TCP stream", &NoneError);
        },
    };

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
    let mut requested_resource: &str = header_first[1];

    if requested_resource == "/" {
        requested_resource = "index.html";
    } else if requested_resource == "/favicon.ico" {
        requested_resource = "images/favicon.png";
    } else if requested_resource.contains("/search?") {
        requested_resource = "search.html";
    };

    // TODO get return MIME type
    let requested_split: Vec<&str> = requested_resource.split(".").collect();
    let content_type: &str = get_mime_type(requested_split[requested_split.len() - 1]);

    let mut status: String = "200 OK".to_string();
    // Note: Any file located in /site/ can be read and transmitted
    let resource: Vec<u8> = match fs::read("site/".to_owned() + requested_resource) {
        Ok(r) => r,
        Err(_) => {
            error::<NoneError>(&("Couldn't provide resource ".to_owned() + requested_resource), &NoneError);

            let r: Vec<u8>;
            if content_type == "text/html; charset=utf-8" {
                (status, r) = ResourceError::not_found_html();
            } else {
                (status, r) = ("404 Not Found".to_string(), ResourceError::not_found_generic());
            };
            // not_found_html may return status 404 or 500, depending on whether the 404.html page can be opened

            r
        },
    };

    let mut response: Vec<u8> = vec![];

    response = ("HTTP/1.1 ".to_owned() + &status + "\r\n").as_bytes().to_vec();

    response.add_header("Server", "Astra");
    response.add_header("Content-Type", content_type);
    //response.add_header("Cache-Control", "max-age=600");

    response.add_header("Content-Length", &resource.len().to_string());
    response.add_body(resource);

    response
}

fn get_mime_type(extension: &str) -> &str {
    match extension {
        // Page
        "html" | "htm" => "text/html; charset=utf-8",
        "js" => "text/javascript",
        "css" => "text/css",
        // Images
        "png" | "ico" => "image/png", // favicon.ico must be a PNG
        "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        // Text
        "txt" | "md" => "text/plain; charset=utf-8",
        // Fallback
        _ => "application/octet-stream",
    }
}

struct ResourceError {}

trait ErrorResponse {
    fn not_found_html() -> (String, Vec<u8>);

    fn not_found_generic() -> Vec<u8>;

    fn internal_error_hmtl() -> Vec<u8>;
}

impl ErrorResponse for ResourceError {
    fn not_found_html() -> (String, Vec<u8>) {
        match fs::read("site/404.html") {
            Ok(r) => ("404 Not Found".to_string(), r),
            Err(_) => {
                ("500 Internal Server Error".to_string(), ResourceError::internal_error_hmtl())
            },
        }
    }

    fn not_found_generic() -> Vec<u8> {
        // For scripts and images where providing a default value is not appropriate
        return vec![]
    }

    fn internal_error_hmtl() -> Vec<u8> {
        // Provices a hard coded 500 for events where 404.html is unavaliable, or other situations where
        // only the precompiled server code can be considered safe
        return ("<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>500 - Internal server error</title>
</head>
<body>
    <h1>500 - Internal Server Error</h1>

    <p>Error 500 - The server was unable to respond to the request</p>
</body>
</html>").as_bytes().to_vec()
    }
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