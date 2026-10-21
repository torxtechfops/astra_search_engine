use std::{
    net::{
        AddrParseError,
        TcpListener,
    },
    vec,
};
use tokio::runtime::{
    Runtime,
    Builder,
};

mod config;
mod error;
mod tcp;

use config::{
    Config,
    FromFile,
};

use crate::{config::{BindSocket, FileExistsError, FileNotFoundError, NewFile}, error::{NoneError, error, fatal_error}, tcp::handle_listener};

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let config_path = "astra.conf";

    let config: Config;
    match Config::from_file(config_path) {
        Ok(c) => {
            config = c;
        },
        Err(e) => { // Create a new config file, then load from that
            error::<FileNotFoundError>("Unable to open config file, attempting to create new file", &e);

            match Config::new_file(config_path) {
                Ok(_) => (),
                Err(e) => {
                    fatal_error::<FileExistsError>("Unable to create default config file", &e);

                    unreachable!();
                }
            };

            config = Config::from_file(config_path).expect("Config file should be avaliable now, as it has been created.");
        }
    };

    // TODO Multiple listeners
    let (addresses, ports): (usize, usize) = config.get_socket_numbers();
    println!("Add: {} | Port: {}", addresses, ports);

    let mut listeners_vec: Vec<TcpListener> = vec![];
    for address in 0..addresses {
        for port in 0..ports {
            listeners_vec.push(
                match TcpListener::bind(config.get_bind_socket(address, port)) {
                    Ok(t) => t,
                    Err(e) => {
                        fatal_error::<AddrParseError>(&("Unable to bind to socket ".to_string() + &config.get_bind_socket(0, 0).to_string()), &e);

                        unreachable!();
                    },
                }
            );
        };
    };

    // Create TCP subhandlers
    let mut handlers_vec: Vec<Runtime> = vec![];
    let mut index: i32 = 0;
    let index_max:usize = listeners_vec.len();
    for listener in listeners_vec {
        index += 1;
        handlers_vec.push({
            let runtime: Runtime = match Builder::new_multi_thread()
                .worker_threads(2)
                .thread_name(
                    listener.local_addr()
                            .unwrap_or(tcp::default_socket())
                            .to_string()
                        )
                .build() {
                Ok(rt) => rt,
                Err(_) => {
                    fatal_error::<NoneError>("Unable to create runtime", &NoneError);
                    
                    unreachable!()
                },
            };

            let index_string: String = index.to_string();
            let max_string: String = index_max.to_string();
            runtime.spawn(async move {handle_listener( &(index_string + "/" + &max_string), listener).await});

            runtime
        });
    };

    Ok(())
}
