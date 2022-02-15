use std::num::NonZeroU32;
use std::{path::Path, fs::File, io::Read};
use async_std::task::spawn;
use futures::StreamExt;
use futures::executor::block_on;
use image::png::PngEncoder;
use imageproc::rect::Rect;
use toml;
use serde::Deserialize;
use std::io::{Error, BufWriter, Write};
use std::thread;
use std::time::{Duration, Instant};
use std::net::{TcpListener, TcpStream, Shutdown, IpAddr};
use async_std::prelude::*;
use async_std::net::{TcpStream as AsyncTcpStream, TcpListener as AsyncTcpListener, Shutdown as AsyncShutdown};
use photon_rs::native::{open_image, open_image_from_bytes, save_image, image_to_bytes};
use photon_rs::transform::{resize, SamplingFilter};
use imageproc::drawing::{self, Canvas};
use imageproc::utils::load_image_or_panic;
use image::{Rgba, Rgb};
use image::io::Reader as ImageReader;
use fast_image_resize as fr;

#[derive(Deserialize)]
struct AppConfig {
    host: IpAddr,
    port: u16,
    password: String,
}

fn resize_image() {
    let input = Path::new("pax.png");
    let output = Path::new("output.png");
    let w_target = 20480;
    let h_target = 20480;

    // Open the image
    let img = ImageReader::open(input).unwrap().decode().unwrap();

    // Must use NonZeroU32 for width and height
    let width = NonZeroU32::new(img.width()).unwrap();
    let height = NonZeroU32::new(img.height()).unwrap();

    // Create a new image buffer from the reader
    let mut src_image = fr::Image::from_vec_u8(width, height, img.to_rgba8().into_raw(), fr::PixelType::U8x4).unwrap();

    let dst_width = NonZeroU32::new(w_target).unwrap();
    let dst_heigth = NonZeroU32::new(h_target).unwrap();

    // Create a new empty image
    let mut dst_image = fr::Image::new(dst_width, dst_heigth, src_image.pixel_type());

    let mut dst_view = dst_image.view_mut();

    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Nearest);
    println!("resizing...");
    let now = Instant::now();
    resizer.resize(&src_image.view(), &mut dst_view).unwrap();
    println!("time = {}", now.elapsed().as_millis());

    let mut img_bytes = Vec::new();
    PngEncoder::new(&mut img_bytes)
    .encode(dst_image.buffer(), dst_width.get(), dst_heigth.get(), image::ColorType::Rgba8)
    .unwrap();

    // let mut file = File::create(output).unwrap();

    let mut stream = File::open("/dev/null").unwrap();

    stream.write_all(&img_bytes).unwrap();
    // file.write_all(&img_bytes).unwrap();

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

    resize_image();
    return;

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

    // Read the config file
    let config_as_string = read_file(config_file).expect("Could not read config file");

    // Deserialize the config file content to our AppConfig struct
    let config: AppConfig = toml::from_str(&config_as_string).unwrap();

    // Panic if the config file is not valid
    assert!(config.host.is_ipv4(), "Host is not set");
    assert!(config.password.len() > 0, "Password is not set");

    // return the config struct
    config
}