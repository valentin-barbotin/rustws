use std::{path::Path, fs::File, io::Read};
use toml;
use serde::Deserialize;
use std::io::Error;

#[derive(Deserialize)]
struct AppConfig {
    host: String,
    password: String,
}

fn main() {
    let config = read_config_file();
    println!("Host value = {host}\nPassword value = {password}", host = config.host, password = config.password);
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