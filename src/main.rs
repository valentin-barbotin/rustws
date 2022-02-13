use std::{path::Path, fs::File, io::Read};
use async_std::task::spawn;
use futures::StreamExt;
use futures::executor::block_on;
use toml;
use serde::Deserialize;
use std::io::{Error, Write};
use std::thread;
use std::time::Duration;
use std::net::{TcpListener, TcpStream, Shutdown};
use async_std::prelude::*;
use async_std::net::{TcpStream as AsyncTcpStream, TcpListener as AsyncTcpListener, Shutdown as AsyncShutdown};
#[derive(Deserialize)]
struct AppConfig {
    host: String,
    password: String,
}

async fn handle_client(mut stream: AsyncTcpStream) {

    let mut buffer = [0; 1024];

    // stream.read(&mut buffer).await.unwrap();

    stream.write_all(b"hello\n").await.unwrap();

    thread::sleep(Duration::from_secs(15));

    stream.write_all(b"END\n").await.unwrap();
    stream.flush().await.unwrap();
}

#[async_std::main]
async fn main() {
    let config = read_config_file();
    println!("Host value = {host}\nPassword value = {password}", host = config.host, password = config.password);

    let listener = AsyncTcpListener::bind("127.0.0.1:4000").await.unwrap();

    listener
    .incoming()
    .for_each_concurrent(None, 
    |stream| async move {
        let stream = stream.unwrap();
        spawn(handle_client(stream));
    }).await;
}

fn read_file(path: &Path) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    Ok(contents)
}

fn read_config_file() -> AppConfig {
    let config_file = Path::new("./config/config.toml");

    let config_as_string = read_file(config_file).unwrap_or_else(|op| {
        panic!("Could not read config file: {}", op);
    });
    let config: AppConfig = toml::from_str(&config_as_string).unwrap();

    assert!(config.host.len() > 0, "Host is not set");
    assert!(config.password.len() > 0, "Password is not set");

    config
}