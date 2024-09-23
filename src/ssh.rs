#![cfg(feature = "ssh")]

use crate::Engine;
use openssh::{Child, Error as SSHError, Stdio};
pub use openssh::{KnownHosts, Session};
use std::{
    io::{Error as IOError, Result as IOResult},
    path::PathBuf,
    pin::Pin,
    str::FromStr,
    time::Duration,
};
use tokio::io::{stdin, AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

/// An SSH session.
pub struct SSH<'a>(pub Child<&'a Session>, String);

impl<'a> SSH<'a> {
    /**
    Launches an executable `file` using a remote [`session`](Session). Returns an [`Engine`]
    connected to that process.
    */
    pub async fn new(session: &'a Session, file: &str) -> Result<Self, SSHError> {
        Ok(Self(
            session.shell(file).stdout(Stdio::piped()).stdin(Stdio::piped()).spawn().await?,
            PathBuf::from_str(file).unwrap().file_name().unwrap().to_str().unwrap().to_owned(),
        ))
    }

    /**
    Like [`new`](Self::new), but pauses the process as soon as it launches. Then, uses `pgrep`
    on the remote host to find the process's PID and reports it back to you, and then waits for you
    to press ENTER.
    */
    pub async fn new_leak(session: &'a Session, file: &str) -> Result<Self, SSHError> {
        let r = Self::new(session, file).await?;
        r.leak_pid().await;
        Ok(r)
    }

    /// See [`new_leak`](Self::new_leak).
    async fn leak_pid(&self) {
        let grepout = String::from_utf8(
            self.0.session().command("pgrep").arg(&self.1).output().await.unwrap().stdout,
        )
        .unwrap();
        if let Some(v) = {
            let mut ids = grepout.split(|b| b == '\n').collect::<Vec<_>>();
            ids.pop();
            ids.pop()
        } {
            println!("PID is {}. Waiting . . .", v);
            println!("[Press ENTER to continue]");

            BufReader::new(stdin()).read_line(&mut String::new()).await.unwrap();
        }
    }
}

impl AsyncWrite for SSH<'_> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, IOError>> {
        Pin::new(self.0.stdin().as_mut().unwrap()).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), IOError>> {
        Pin::new(self.0.stdin().as_mut().unwrap()).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), IOError>> {
        Pin::new(self.0.stdin().as_mut().unwrap()).poll_shutdown(cx)
    }
}

impl AsyncRead for SSH<'_> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<IOResult<()>> {
        Pin::new(self.0.stdout().as_mut().unwrap()).poll_read(cx, buf)
    }
}

impl Engine for SSH<'_> {
    const TIMEOUT: Duration = Duration::from_millis(50);
    const REPEAT: usize = 3;
}
