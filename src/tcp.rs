#![cfg(feature = "tcp")]

use crate::Engine;
use std::time::Duration;
use tokio::net::TcpStream;

impl Engine for TcpStream {
    const TIMEOUT: Duration = Duration::from_millis(50);
    const REPEAT: usize = 5;
}

/// Shorthand to open a TCP connection [using tokio](tokio::net::TcpStream).
pub async fn tcp(url: &'static str) -> std::io::Result<TcpStream> {
    Ok(TcpStream::connect(url).await?)
}
