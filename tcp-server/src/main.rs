// Homework requires all statements to be commented

// use tokio for async runtime
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

// use anyhow for error handling
use anyhow::{bail, Context, Result};

// type for socket address
use std::net::SocketAddr;

const BUFFER_SIZE: usize = 1024;

/// handle a TCP stream from client
async fn handle_client(mut socket: TcpStream, client_address: SocketAddr) -> Result<u64> {
    // buffer for incoming message
    let mut buffer = [0u8; BUFFER_SIZE];
    // split socket
    let (mut reader, mut writer) = socket.split();
    // total count of received bytes
    let mut echoed_bytes_count: u64 = 0;

    // loop to handle data
    loop {
        match reader.read(&mut buffer).await {
            // connection closed
            Ok(0) => return Ok(echoed_bytes_count),
            // receive message
            Ok(n) => {
                // add to sum
                echoed_bytes_count += n as u64;

                // print to screen
                println!(
                    "From {:?}: {}",
                    client_address,
                    String::from_utf8_lossy(&buffer)
                );

                // echo
                writer
                    .write(&buffer)
                    .await
                    .context("Failed to write to client")?;
            }
            Err(err) => bail!("Failed to echo from {:?}: {}", client_address, err),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // initialize a TCP socket server
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .context("Failed to initialize TCP server")?;

    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 8080");

    loop {
        // try to accept an incoming connection
        match listener.accept().await {
            // when connection established
            Ok((socket, client_address)) => {
                // spawn a new task to handle this connection
                tokio::spawn(async move {
                    println!("New connection from {:?}", client_address);
                    match handle_client(socket, client_address).await {
                        // connection closed
                        Ok(n) => {
                            println!(
                                "Connection from {:?} closed, with {} bytes echoed",
                                client_address, n
                            );
                        }
                        // error happened
                        Err(e) => {
                            println!("{}", e);
                        }
                    }
                });
            }
            // when connection failed to be established
            Err(e) => {
                println!("Failed to establish a connection: {}", e);
            }
        }
    }
}
