use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use base64::{engine::general_purpose, Engine as _};

use crate::chromium::chromium_manager::ChromiumProfile;

#[derive(Clone)]
pub struct ProxyServer {
    // pub local_addr: String,
}

impl ProxyServer {
    pub fn new() -> Self {
        Self {
            // local_addr: String::new(),
        }
    }

    pub fn start_server_proxy(&self, profile: &ChromiumProfile ) -> Result<(), Box<dyn std::error::Error>> {
        let server_proxy = self.clone();
        let profile_to_proxify = profile.clone();
        tokio::spawn(async move {
            match server_proxy.server_proxy(profile_to_proxify).await {
                Ok(_) => println!("Proxy server started successfully."),
                Err(e) => eprintln!("Error starting proxy server: {}", e),
            }
        });
        Ok(())
    }

    pub async fn server_proxy(&self, profile: ChromiumProfile) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", profile.proxy_server_port)).await?;

        println!("Listening on {}", listener.local_addr().unwrap());

        loop {
            // Accepts incoming connections
            let (mut client_socket, client_addr) = listener.accept().await?;
            println!("Incoming connection from: {}", client_addr);

            let tokio_profile = profile.clone();
            tokio::spawn(async move {
                // Reads the initial request from the client
                // (ex. "CONNECT target:port HTTP/1.1")
                let mut buffer = [0u8; 1024];
                let n = match client_socket.read(&mut buffer).await {
                    Ok(n) if n > 0 => n,
                    _ => {
                        eprintln!("Error while reading connection response from client");
                        return;
                    }
                };

                let req_str = String::from_utf8_lossy(&buffer[..n]);
                let mut lines = req_str.lines();
                let request_line = match lines.next() {
                    Some(line) => line,
                    None => {
                        eprintln!("Empty request from client");
                        return;
                    }
                };

                // The command CONNECT should be like "CONNECT target:port HTTP/1.1"
                let parts: Vec<&str> = request_line.split_whitespace().collect();
                if parts.len() < 3 {
                    eprintln!("Invalid format request: {}", request_line);
                    return;
                }
                let target_addr = parts[1];
                println!("Connection request to: {}", target_addr);

                // Remote proxy address
                let remote_proxy_addr = format!("{}:{}", tokio_profile.proxy.host.as_str(), tokio_profile.proxy.port);

                // Connection to the remote proxy
                match TcpStream::connect(remote_proxy_addr.clone()).await {
                    Ok(mut remote_socket) => {
                        // Basic Authentication Handler
                        let username = format!("{}", tokio_profile.proxy.username);
                        let password = format!("{}", tokio_profile.proxy.password);
                        let credentials = format!("{}:{}", username, password);
                        let auth_base64 = general_purpose::STANDARD.encode(credentials);

                        // Builds the CONNECT request to send to the remote proxy
                        let connect_req = format!(
                            "CONNECT {} HTTP/1.1\r\nHost: {}\r\nProxy-Authorization: Basic {}\r\n\r\n",
                            target_addr, target_addr, auth_base64
                        );
                        if let Err(e) = remote_socket.write_all(connect_req.as_bytes()).await {
                            eprintln!("Error sending CONNECT command to the remote proxy: {}", e);
                            return;
                        }

                        // Reads the response from the remote proxy
                        let mut remote_buffer = [0u8; 1024];
                        let n = match remote_socket.read(&mut remote_buffer).await {
                            Ok(n) if n > 0 => n,
                            _ => {
                                eprintln!("No response from the remote proxy");
                                return;
                            }
                        };
                        let response_str = String::from_utf8_lossy(&remote_buffer[..n]);
                        println!("Response from the remote proxy: {}", response_str);

                        // Checks whether the response indicates the connection was successful
                        if !response_str.starts_with("HTTP/1.1 200") {
                            eprintln!("The remote proxy refused the connection or authentication failed");
                            // Optionally forwards the response to the client to inform it
                            let _ = client_socket.write_all(&remote_buffer[..n]).await;
                            return;
                        }

                        // Sends a response to the client (Chrome) confirming the connection was established
                        let client_response = b"HTTP/1.1 200 Connection established\r\n\r\n";
                        if let Err(e) = client_socket.write_all(client_response).await {
                            eprintln!("Error sending response to client: {}", e);
                            return;
                        }

                        // Starts bidirectional forwarding between the client and the remote proxy
                        let (mut client_reader, mut client_writer) = client_socket.into_split();
                        let (mut remote_reader, mut remote_writer) = remote_socket.into_split();

                        let client_to_remote = io::copy(&mut client_reader, &mut remote_writer);
                        let remote_to_client = io::copy(&mut remote_reader, &mut client_writer);

                        match tokio::try_join!(client_to_remote, remote_to_client) {
                            Ok((bytes_c2r, bytes_r2c)) => {
                                println!(
                                    "Connection closed: {} bytes sent -> {}, {} bytes received <- {}",
                                    bytes_c2r, target_addr, bytes_r2c, target_addr
                                );
                            }
                            Err(e) => {
                                eprintln!("Error during forwarding: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Unable to connect to remote proxy {}: {}", remote_proxy_addr, e);
                    }
                }
            });
        }
    }
}