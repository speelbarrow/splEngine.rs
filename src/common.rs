#![cfg(any(feature = "tcp", feature = "ssh"))]

use std::{error::Error, future::Future, string::FromUtf8Error, time::Duration};
use tokio::{
    io::{stdout, AsyncReadExt, AsyncWriteExt, Error as IOError},
    join,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::timeout,
};

/// A remote stream that takes input.
#[trait_variant::make(Send)]
pub trait Engine: AsyncReadExt + AsyncWriteExt + Unpin + Sized {
    const TIMEOUT: Duration;
    const REPEAT: usize = 1;

    /// Reads the last chunk. See [`read_chunk`](Engine::read_chunk)
    async fn read_last_chunk(&mut self) -> Result<String, FromUtf8Error> {
        async {
            let mut buf = Vec::new();
            let mut dropped = vec![false; Self::REPEAT];
            'a: loop {
                match timeout(Self::TIMEOUT, self.read_u8()).await {
                    Ok(Ok(b)) => {
                        dropped = vec![false; Self::REPEAT];
                        buf.push(b);
                    }
                    _ => {
                        for i in 0..Self::REPEAT {
                            if !dropped[i] {
                                dropped[i] = true;
                                continue 'a;
                            }
                        }
                        return Ok(String::from_utf8(buf)?);
                    }
                }
            }
        }
    }

    /**
    Reads one "chunk" of remote input. A chunk "ends" when no new data is received for
    [`TIMEOUT`](Engine::TIMEOUT) amount of time. This does not apply to the first byte read -- the
    function will wait indefinitely until it receives *some* data from the remote stream.
    */
    async fn read_chunk(&mut self) -> Result<String, FromUtf8Error> {
        async {
            Ok(String::from_utf8(vec![self.read_u8().await.unwrap()])?
                + &self.read_last_chunk().await?)
        }
    }

    /**
    Like [`run`](Engine::run), but forwards input received from the remote process over an
    [unbounded channel](tokio::sync::mpsc::unbounded_channel).
    */
    fn run_with_channel<'a, I>(
        &mut self,
        input: I,
    ) -> (
        UnboundedReceiver<String>,
        impl Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send,
    )
    where
        I: IntoIterator<Item = &'a [u8]> + Send,
        <I as IntoIterator>::IntoIter: Send,
    {
        let (sender, receiver) = unbounded_channel();
        let future = async move {
            let mut stdout = stdout();
            for i in input {
                write(self.read_chunk().await?, Some(&sender)).await?;
                let (r1, r2) = join!(self.write_all(i), async {
                    stdout.write_all(i).await?;
                    stdout.write_u8(b'\n').await?;
                    Ok::<(), IOError>(())
                });
                r1?;
                r2?;
            }

            let chunk = self.read_last_chunk().await?;
            for string in chunk.split("\n") {
                if !sender.is_closed() {
                    sender.send(string.to_owned()).unwrap();
                }
            }
            stdout.write_all(chunk.as_bytes()).await?;
            Ok(())
        };
        (receiver, future)
    }

    /**
    Executes a series of transactions as such:
    1. Wait for data from the remote stream (see [`read_chunk`](Engine::read_chunk))
    2. Pops one `&[u8]` from the top of `input` and writes it to the remote stream.
    3. Repeat.
    */
    async fn run<'a, I>(&mut self, input: I) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        I: IntoIterator<Item = &'a [u8]> + Send,
        <I as IntoIterator>::IntoIter: Send,
    {
        self.run_with_channel(input).1
    }
}

async fn write(
    chunk: String,
    sender: Option<&UnboundedSender<String>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(sender) = sender {
        if !sender.is_closed() {
            for part in chunk.split("\n") {
                sender.send(part.to_owned())?;
            }
        }
    }
    Ok(stdout().write_all(chunk.as_bytes()).await?)
}

/**
Shorthand for commonly used connection semantics.

Recommended recipe:
```no_run
use engine::{connect, ssh::{Session, KnownHosts}};

# #[tokio::main]
# async fn main() {
#[allow(unused)]
let session = Session::connect_mux("remote.host.org", KnownHosts::Strict).await.unwrap();
// Remove the comment on this line ...
let mut handle = // connect!(@ssh session, "/path/to/executable").await.unwrap();
    connect!(@tcp "www.example.com:65535").await.unwrap(); // ... and comment out this line ...
                                                           // to "switch modes".
# }
```
*/
#[macro_export]
macro_rules! connect {
    (@tcp $url: literal) => {{
        engine::tcp($url)
    }};
    (@ssh $session: ident, $file: literal) => {{
        engine::SSH::new_leak(&$session, $file)
    }};
}
