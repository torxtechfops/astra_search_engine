pub trait RequestBase {
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

pub enum RequestType {
    Resource, // A page, script, or other web related document already stored is requested
    Cache, // A cache search is requested
    Search, // A search engine query is requested
}