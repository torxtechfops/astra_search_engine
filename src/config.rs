use std::{
    fmt, 
    fs::{
        self, 
        File}, 
        io::{
            Error, 
            Write
        }, 
    net::{
        AddrParseError, 
        IpAddr, 
        SocketAddr,
    }, 
    str::FromStr
};
use regex::Regex;

use crate::{
    config::SettingType::{
        CriticalDuplicate, 
        CriticalUnique, 
        Duplicate, 
        Unique
    }, 
    error::{
        error, 
        fatal_error,
    },
};

#[allow(dead_code)]
pub enum SettingType {
    CriticalUnique, // Setting must have only one value
    CriticalDuplicate, // Setting must exist, but may have any quantity
    Unique, // Setting can either be absent, or have one associated value
    Duplicate, // Setting may exist or not, in any quantity
    // Note: Reading from multiple setting strings is not supported, multiple values must
    // be read from the same Setting= structure, seperated by a comma (,)
}

pub struct Config {
    address: Vec<IpAddr>,
    port: Vec<u16>,
}

pub trait FromFile {
    fn from_file(path: &str) -> Result<Config, FileNotFoundError>;

    fn get_address_from_string(string: &str) -> IpAddr;

    fn get_setting_value(split_file: &str, search_term: &str, search_type: SettingType) -> Result<Option<String>, RequiredSettingMissingError>;

    fn handle_setting(split_file: &str, search_term: &str, search_type: SettingType) -> Vec<String>;
}

impl FromFile for Config {
    fn from_file(path: &str) -> Result<Config, FileNotFoundError> {
        let mut created_config: Config = Config::default();

        let file_string: String = match fs::read_to_string(path) {
            Ok(f) => f,
            Err(e) => {
                error::<Error>("Unable to load config file", &e);

                return Err(FileNotFoundError)
            }
        };

        created_config.address = {
            let mut interim_vec: Vec<IpAddr> = vec![]; // Mutable placeholder

            Config::handle_setting(&file_string, "Address", CriticalDuplicate)
                .iter()
                .for_each(|x| {interim_vec.push(Config::get_address_from_string(x));});
            // Get string vector of addresses, iterate through vector, convert each to an IP address of IpAddr class, then add to
            // The interim vector

            interim_vec // Final return
        };
        ////println!("{:?}", file_string);
        
        created_config.port = {
            let mut interim_vec: Vec<u16> = vec![];

            Config::handle_setting(&file_string, "Port", CriticalDuplicate)
                .iter()
                .for_each(|x| {
                    interim_vec.push(match x.parse::<u16>() {
                        Ok(int) => int,
                        Err(e) => {
                            fatal_error::<std::num::ParseIntError>(&("Could not parse ".to_owned() + &x + "as a u16 port"), &e);

                            unreachable!();
                        },
                    });
                }
            );

            interim_vec
        };

        return Ok(created_config)
    }

    fn handle_setting(file_string: &str, search_term: &str, search_type: SettingType) -> Vec<String> { // Abstraction to avoid rewriting logic
        match search_type {
            CriticalDuplicate | Duplicate => {
                match Config::get_setting_value(&file_string, search_term, search_type) {
                    Ok(Some(str)) => {
                        let split_str: Vec<&str> = str.split(",").collect();
                        let mut string_vec: Vec<String> = vec![];
                        split_str.iter().for_each(|x| {string_vec.push(x.to_string());});

                        string_vec
                    },
                    Ok(None) => { // This is required so cannot 
                        unreachable!();
                    },
                    Err(e) => {
                        fatal_error::<RequiredSettingMissingError>("Required setting missing", &e);
                        unreachable!();
                    },
                }
            },
            CriticalUnique | Unique => { // 
                match Config::get_setting_value(&file_string, search_term, search_type) {
                    Ok(Some(str)) => {
                        let split_str: Vec<&str> = str.split(",").collect();
                        let string_vec: Vec<String> = vec![split_str[0].to_string()];
                        // No point returning anything extra, the program expects a single
                        // value for this set

                        string_vec
                    },
                    Ok(None) => { // This is required so cannot 
                        unreachable!();
                    },
                    Err(e) => {
                        fatal_error::<RequiredSettingMissingError>("Required setting missing", &e);
                        unreachable!();
                    },
                }
            },
        }
    }

    fn get_address_from_string(string: &str) -> IpAddr {
        return match IpAddr::from_str(string) {
            Ok(r) => r,
            Err(e) => {
                fatal_error::<AddrParseError>(&("Failed to parse IP address from config string (".to_owned() + &string + ")"), &e);
                unreachable!()
            },
        }
    }

    fn get_setting_value(split_file: &str, search_term: &str, search_type: SettingType) -> Result<Option<String>, RequiredSettingMissingError> {
        let regex: Regex = Regex::new(&(r"(".to_owned() + search_term + "{1}=.*)\n")).expect("This \
        should remain a valid RegEx, as it has been tested");
        // This RegEx matches for the search, all values after the term before a new line, and only
        // matches if the match begins on a new line, reducing the risk of matching from a comment

        ////println!("{}", regex);
        
        let captures: regex::Captures<'_> = match regex.captures(split_file) {
            Some(cap) => cap,
            None => {
                match search_type {
                    SettingType::CriticalUnique | SettingType::Unique => {
                        return Err(RequiredSettingMissingError);
                    },
                    _ => {
                        return Ok(None);
                    },
                };
            },
        };

        let mut value: &str = match captures
            .get(0)
            .map_or("", |m| m.as_str())
            .split("=").nth(1) {
                Some(s) => s,
                None => {
                    fatal_error::<RequiredSettingMissingError>("No value associated with setting ", &RequiredSettingMissingError);
                    // Not all settings are required, but if assigned, a value is always required

                    unreachable!();
                },
            };

        match value.strip_suffix('\n') { // Returns a result, this is handled to avoid removing a value at the end of a file
            Some(s) => {
                value = s;
            },
            None => {},
        };

        ////println!("{}", value);

        return Ok(Some(value.to_string()))
    }
}

pub trait NewFile {
    fn new_file(path: &str) -> Result<(), FileExistsError>;
}

impl NewFile for Config {
    fn new_file(path: &str) -> Result<(), FileExistsError> {
        let file_exists: bool = match fs::exists(&path) {
            Ok(r) => r,
            Err(e) => {
                error::<std::io::Error>("unable to check if config file exists", &e);
                true
            },
        };

        if file_exists {
            return Err(FileExistsError)
        };

        // This also checks for a file existing, the overlap is not an operational concern
        let mut file: File = match File::create_new(&path) {
            Ok(f) => f,
            Err(e) => {
                error::<Error>("File creation error", &e);
                return Err(FileExistsError)
            },
        };

        // Standard config for example purposes
        let standard_config: &str = "\
        # Default config, generated by Astra. This \
        configuration file must be edited to be used \
        normally. Note: config file must end in a new line\n\
        Address=127.0.0.1\n\
        Port=80,443\n"; // TODO Link with default config

        match file.write(standard_config.as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                fatal_error::<std::io::Error>("Unable to write new config file", &e);
            }
        }

        return Ok(())
    }
}

pub trait BindSocket {
    fn get_bind_socket(&self, address_index: usize, port_index: usize) -> SocketAddr;

    fn get_socket_numbers(&self) -> (usize, usize);
}

impl BindSocket for Config {
    fn get_bind_socket(&self, address_index: usize, port_index: usize) -> SocketAddr {
        return (self.address[address_index], self.port[port_index]).into()
    }

    fn get_socket_numbers(&self) -> (usize, usize) {
        return (self.address.len(), self.port.len());
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            address: vec![IpAddr::from_str("127.0.0.1").expect("This is a built-in default and should not fail")],
            port: vec![80_u16, 443_u16],
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileExistsError;

impl fmt::Display for FileExistsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "file with the provided path already exists")
    }
}

impl std::error::Error for FileExistsError {}

#[derive(Debug, Clone)]
pub struct FileNotFoundError;

impl fmt::Display for FileNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "file with the provided path does not exist")
    }
}

impl std::error::Error for FileNotFoundError {}

#[derive(Debug, Clone)]
pub struct RequiredSettingMissingError;

impl fmt::Display for RequiredSettingMissingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "config file is missing at least one required setting")
    }
}

impl std::error::Error for RequiredSettingMissingError {}