use std::fmt::format;
use std::{path::Path, fs::File, io::Read};
use async_std::task::spawn;
use futures::StreamExt;
use futures::executor::block_on;
use toml;
use serde::Deserialize;
use std::io::{Error, Write};
use std::thread;
use std::time::Duration;
use std::net::{TcpListener, TcpStream, Shutdown, IpAddr};
use async_std::prelude::*;
use async_std::net::{TcpStream as AsyncTcpStream, TcpListener as AsyncTcpListener, Shutdown as AsyncShutdown};
use photon_rs::native::{open_image, open_image_from_bytes, save_image, image_to_bytes};
use photon_rs::transform::{resize, SamplingFilter};

#[derive(Deserialize)]
struct AppConfig {
    host: IpAddr,
    port: u16,
    password: String,
}

fn resize_image() {
    let mut img = open_image("photo.jpg").unwrap();

    let width = 10240;
    let heigth = 10240;
    img = resize(&img, width, heigth, SamplingFilter::Nearest);

    save_image(img, "output2.jpg");
    println!("resize done");
}


/// Handle a single client connection.
async fn handle_client(mut stream: AsyncTcpStream) {

    let mut buffer = [0; 1024];

    // TODO: use a real password
    let password = "admin";

    stream.write_all(b"Enter the password : ").await.unwrap();

    let n = stream.read(&mut buffer).await.unwrap();
    // Remove the breakline
    let mut content = String::from_utf8(buffer[..n].to_vec()).unwrap();
    content.pop();

    // Check the password
    if content.as_str() != password {
        stream.write_all(b"Wrong password\n").await.unwrap();
        stream.flush().await.unwrap();
        return;
    }

    while match stream.read(&mut buffer).await {
        Ok(n) => {
            if n == 0 {
                return;
            }

            content = String::from_utf8(buffer[..n].to_vec()).unwrap();
            // Remove the breakline
            content.pop();

            // If the user type
            // exit: we close the connection
            // hello: we send a hello message
            // other: we send default message
            match content.as_str() {
                "hello" => {
                    stream.write_all(b"Yes yes !\n").await.unwrap();
                },
                "exit" => {
                    stream.shutdown(Shutdown::Both).unwrap();
                }
                _ => {
                    stream.write_all(b"default\n").await.unwrap();
                }
            }
            true
        },
        Err(e) => {
            println!("{}", e);
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {};
    stream.flush().await.unwrap();
}

/// This is the main function
/// async_std allows to use async functions from main thread
#[async_std::main]
async fn main() {
    let config = read_config_file();
    println!("Host value = {host}\nPassword value = {password}", host = config.host, password = config.password);

    let address = format!("{}:{}", config.host, port = config.port);
    let listener = AsyncTcpListener::bind(address).await.unwrap();

    listener
    .incoming()
    .for_each_concurrent(None, 
    |stream| async move {
        let stream = stream.unwrap();
        spawn(handle_client(stream));
    }).await;
}

/// Read a file and return its content as a String
fn read_file(path: &Path) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    Ok(contents)
}

/// Read the config file and return its content as a AppConfig struct
fn read_config_file() -> AppConfig {
    let config_file = Path::new("./config/config.toml");

    /// Read the config file
    let config_as_string = read_file(config_file).expect("Could not read config file");

    /// Deserialize the config file content to our AppConfig struct
    let config: AppConfig = toml::from_str(&config_as_string).unwrap();

    /// Panic if the config file is not valid
    assert!(config.host.is_ipv4(), "Host is not set");
    assert!(config.password.len() > 0, "Password is not set");

    ///return the config struct
    config
}