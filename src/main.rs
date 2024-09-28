use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
// use tokio::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::rc::Rc;

// macro_rules! path_group {
//     ($buffer:expr, $prefix:expr) => {
//         $buffer.starts_with($prefix)
//     };
// }

macro_rules! path_group {
    ($buffer:expr, $request:expr, $block:block) => {
        // Here you can do something with $buffer and $request
        if $buffer.starts_with($request) {
            // Execute the block of code
            $block
        }
    };
}

macro_rules! staticFile {
    ($hashmap:expr, $path:expr, $ft:expr) => {
        let empty: &str = "lul no";
        let mut content: Vec<u8> = empty.as_bytes().to_vec();
        if cfg!(debug_assertions) {
            content = fs::read(concat!("src/site", $path)).unwrap_or_else(|_| vec![]);
        } else {
            content = include_bytes!(concat!("site", $path)).to_vec();
            // .map(|s| String::from(s))
        }
        $hashmap.insert($path, (content, String::from($ft)))
    };
}

async fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];
    // Read the request
    let bytes_read = stream.read(&mut buffer).await?;

    // Convert the request to a content
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Received request:\n{}", request);

    path_group!(buffer, b"GET / HTTP/1.1\r\n", {
        let content: String;
        if cfg!(debug_assertions) {
            // println!("Build type: Debug");
            content = fs::read_to_string("src/site/index.html")
                .unwrap_or_else(|_| String::from("lul no"));
        } else {
            // println!("Build type: Release");
            content = String::from(
                std::str::from_utf8(include_bytes!("site/index.html")).unwrap_or_else(|_| ""),
            );
            // .map(|s| String::from(s))
        }

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            content.len(),
            content
        );
        println!("---response: {}", response);
        stream.write(response.as_bytes()).await?;
    });

    path_group!(buffer, b"GET /assets/", {
        let mut paths: HashMap<&str, (Vec<u8>, String)> = HashMap::new();

        staticFile!(paths, "/assets/style.css", "text/css");
        staticFile!(paths, "/assets/style.css", "text/css");
        staticFile!(paths, "/assets/fonts/Roboto_Mono/static/RobotoMono-Regular.ttf", "font/ttf");
        staticFile!(paths, "/assets/fonts/Roboto_Mono/static/RobotoMono-Bold.ttf", "font/ttf");
        staticFile!(paths, "/assets/js/main.js", "application/javascript");
        staticFile!(paths, "/assets/js/fs.js", "application/javascript");
        staticFile!(paths, "/assets/js/toggle-visible.js", "application/javascript");
        staticFile!(paths, "/assets/js/toggle-qr.js", "application/javascript");
        staticFile!(paths, "/assets/wasm/load.js", "application/javascript");
        staticFile!(paths, "/assets/img/plik.svg", "image/svg+xml");
        staticFile!(paths, "/assets/img/plus.svg", "image/svg+xml");

        let byte_buffer: Vec<u8> = buffer.iter().map(|&bite| bite as u8).collect();
        let path_raw = std::str::from_utf8(&byte_buffer).unwrap();
        let empty: (Vec<u8>, String) = (vec![], String::from(""));
        let path_correct: Vec<&str> = path_raw.splitn(3, ' ').collect();
        // println!("path split: {}", path_correct[1]);
        
        //[ returning content and mime type from hashmap key ]
        let (content, ft) = paths.get(path_correct[1]).unwrap_or(&empty);

        let response = [
            b"HTTP/1.1 200 OK\r\nContent-Type: ",
            ft.as_bytes(),
            b"\r\nContent-Length: ",
            content.len().to_string()[..].as_bytes(),
            b"\r\n\r\n",
            content,
        ]
        .concat();
        // println!("---response: {}", response);
        stream.write(&response).await?;
    });

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Bind the TCP listener to 0.0.0.0:3000
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    if cfg!(debug_assertions) {
        println!("Build type: Debug");
    } else {
        println!("Build type: Release");
    }
    println!("Server running on http://0.0.0.0:3000");

    loop {
        // Accept incoming connections
        let (stream, _) = listener.accept().await?;

        // Spawn a new task to handle the connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Failed to handle connection: {}", e);
            }
        });
    }
}
