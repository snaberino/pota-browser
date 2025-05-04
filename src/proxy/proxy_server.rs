use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use base64::{engine::general_purpose, Engine as _};

use crate::chromium::chromium_manager::ChromiumProfile;

use tokio_socks::tcp::Socks5Stream;

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
        println!("Proxy listening on {}", listener.local_addr()?);
    
        loop {
            let (mut client_socket, client_addr) = listener.accept().await?;
            println!("Incoming connection from: {}", client_addr);
            let tokio_profile = profile.clone();
    
            tokio::spawn(async move {
                let mut buffer = [0u8; 1024];
                let n = match client_socket.read(&mut buffer).await {
                    Ok(n) if n > 0 => n,
                    _ => {
                        eprintln!("Error reading from client");
                        return;
                    }
                };
    
                let req_str = String::from_utf8_lossy(&buffer[..n]);
                let mut lines = req_str.lines();
                let request_line = match lines.next() {
                    Some(line) => line,
                    None => {
                        eprintln!("Empty request");
                        return;
                    }
                };
    
                let parts: Vec<&str> = request_line.split_whitespace().collect();
                if parts.len() < 3 {
                    eprintln!("Invalid request format: {}", request_line);
                    return;
                }
    
                let target_addr = parts[1];
                let target_host = target_addr.split(':').next().unwrap_or("localhost");
                let target_port = target_addr.split(':').nth(1).unwrap_or("80").parse::<u16>().unwrap_or(80);
    
                let proxy = &tokio_profile.proxy;
                let remote_socket: TcpStream;
    
                if proxy.protocol == "socks5" {
                    match Socks5Stream::connect_with_password(
                        (proxy.host.as_str(), proxy.port.parse::<u16>().unwrap_or(1080)),
                        (target_host, target_port),
                        &proxy.username,
                        &proxy.password
                    ).await {
                        Ok(s) => {
                            remote_socket = s.into_inner();
                            println!("SOCKS5 tunnel established to {}:{}", target_host, target_port);
                        }
                        Err(e) => {
                            eprintln!("SOCKS5 connection error: {}", e);
                            return;
                        }
                    }
                } else if proxy.protocol == "http" {
                    match TcpStream::connect(format!("{}:{}", proxy.host, proxy.port)).await {
                        Ok(mut s) => {
                            let credentials = format!("{}:{}", proxy.username, proxy.password);
                            let auth_base64 = general_purpose::STANDARD.encode(credentials);
                            let connect_req = format!(
                                "CONNECT {} HTTP/1.1\r\nHost: {}\r\nProxy-Authorization: Basic {}\r\n\r\n",
                                target_addr, target_addr, auth_base64
                            );
                            if let Err(e) = s.write_all(connect_req.as_bytes()).await {
                                eprintln!("Failed to send CONNECT request: {}", e);
                                return;
                            }
    
                            let mut resp_buf = [0u8; 1024];
                            let n = match s.read(&mut resp_buf).await {
                                Ok(n) if n > 0 => n,
                                _ => {
                                    eprintln!("No response from HTTPS proxy");
                                    return;
                                }
                            };
    
                            let response_str = String::from_utf8_lossy(&resp_buf[..n]);
                            if !response_str.starts_with("HTTP/1.1 200") {
                                eprintln!("HTTPS proxy refused connection:\n{}", response_str);
                                let _ = client_socket.write_all(&resp_buf[..n]).await;
                                return;
                            }
    
                            remote_socket = s;
                            println!("HTTPS tunnel established to {}", target_addr);
                        }
                        Err(e) => {
                            eprintln!("Failed to connect to HTTPS proxy: {}", e);
                            return;
                        }
                    }
                } else {
                    eprintln!("Unsupported proxy protocol: {}", proxy.protocol);
                    return;
                }
    
                // Conferma al client (Chromium)
                let client_response = b"HTTP/1.1 200 Connection established\r\n\r\n";
                if let Err(e) = client_socket.write_all(client_response).await {
                    eprintln!("Failed to send response to client: {}", e);
                    return;
                }
    
                // Forwarding dati tra client ↔ proxy
                let (mut client_reader, mut client_writer) = client_socket.into_split();
                let (mut remote_reader, mut remote_writer) = remote_socket.into_split();
    
                let client_to_remote = io::copy(&mut client_reader, &mut remote_writer);
                let remote_to_client = io::copy(&mut remote_reader, &mut client_writer);
    
                match tokio::try_join!(client_to_remote, remote_to_client) {
                    Ok((sent, received)) => {
                        println!("Closed connection ({} bytes sent, {} bytes received)", sent, received);
                    }
                    Err(e) => {
                        eprintln!("Forwarding error: {}", e);
                    }
                }
            });
        }
    }
}