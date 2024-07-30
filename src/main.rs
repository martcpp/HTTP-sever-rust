// Uncomment this block to pass the first stage
use anyhow::{Context, Result};
use std::{env, fs};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::{select, signal};
use tokio::fs as file;
use std::fs::OpenOptions;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221")
        .await
        .context("fail to connect to tcp")?;

    loop {
        select! {
           req = listener.accept() => {
            let (stream, _) = req.context("Failed to accept connection")?;
                tokio::spawn(async move{
                let _ = handle_client(stream).await;
            });
            println!("Connection accepted");

            },
            _ = signal::ctrl_c() => {
                println!("Ctrl-C received, shutting down");
                break;

        }

        }
    }

    Ok(())
}

async fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    let len = stream.read(&mut buffer).await?;

    let req_type = String::from_utf8_lossy(&buffer[..len]);
    let line: Vec<&str> = req_type.lines().collect();
    let path = line[0].split_whitespace().nth(1).unwrap();
    let path_split: Vec<&str> = path.split('/').collect();
    let user_agent: Vec<&str> = line[2].split(' ').collect();
    for n in line.iter(){
        println!("{}", n);
    }
 

    if req_type.starts_with("GET / HTTP/1.1") {
        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await.unwrap();
    } else if req_type.starts_with("GET /user-agent HTTP/1.1") {
        let resp = user_agent[1];
        let resp_body = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            resp.len(),
            resp
        );
        stream.write_all(resp_body.as_bytes()).await.unwrap();
    } else if path_split[1] == "echo" && path_split.len() > 2 {
        let resp = path_split[2];
        let resp_body = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            resp.len(),
            resp
        );
        stream.write_all(resp_body.as_bytes()).await.unwrap();
    } else if path_split[1] == "files" && path_split.len() > 2 {
        let arg = env::args().collect::<Vec<String>>();
        let file_name = &path_split[2];
        let mut dir = arg.get(2).unwrap().clone();
        dir.push_str(&file_name);

        if req_type.starts_with("GET") {
        let contents = fs::read_to_string(&dir);
        match contents {
            Ok(content) => {
                let resp_body = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                    content.len(),
                    content
                );
                stream.write_all(resp_body.as_bytes()).await.unwrap();
            }
            Err(e) => {
                println!("Error reading file: {}", e);
                stream
                    .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                    .await
                    .unwrap();
            }
        }
        }else if req_type.starts_with("POST") && line[3].contains("octet-stream"){
            let content_body = line[5];
            println!("he come here");
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&dir).unwrap();
            file.write_all(content_body.as_bytes()).unwrap();

            let response = format!("HTTP/1.1 201 Created\r\n\r\n");
            stream.write_all(response.as_bytes()).await?;
        }else{
            println!("he no enter post he come jumpoooooooooooooooo");
        }

    } else {
        stream
            .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
            .await
            .unwrap();
    }
    Ok(())
}
