use crate::*;
use runtime::RwLock;
pub use runtime::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf, split};
pub use tokio::net::TcpStream as TokioTcpStream;

#[derive(Debug)]
struct TcpStreamWriter {
    connected: bool,
    writer: WriteHalf<TokioTcpStream>,
}

impl TcpStreamWriter {
    fn new(writer: WriteHalf<TokioTcpStream>) -> Self {
        Self {
            connected: true,
            writer,
        }
    }

    pub async fn write(&mut self, buf: &[u8]) -> Result<()> {
        if !self.connected {
            return Unexpected!("write to unconnected stream");
        }
        match self.writer.write_all(buf).await {
            Ok(..) => Ok(()),
            Err(e) => {
                self.connected = false;
                Err(e.into())
            }
        }
    }
}

#[derive(Clone)]
pub struct TcpStream {
    address: String,
    port: u16,
    reader: Arc<RwLock<ReadHalf<TokioTcpStream>>>,
    writer: Arc<RwLock<TcpStreamWriter>>,
}

impl TcpStream {
    pub async fn connect(address: String, port: u16) -> Result<Self> {
        let stream = TokioTcpStream::connect((address.clone(), port)).await?;

        let (reader, writer) = split(stream);

        Ok(Self {
            address,
            port,
            reader: Arc::new(RwLock::new(reader)),
            writer: Arc::new(RwLock::new(TcpStreamWriter::new(writer))),
        })
    }

    pub async fn verify_or_reconnect(&self) -> Result<()> {
        if self.writer.read().await.connected {
            return Ok(());
        }
        let stream = TokioTcpStream::connect((self.address.clone(), self.port)).await?;
        let (reader, writer) = split(stream);
        {
            let mut reader_lock = self.reader.write().await;
            *reader_lock = reader;
        }
        {
            let mut writer_lock = self.writer.write().await;
            *writer_lock = TcpStreamWriter::new(writer);
        }
        Ok(())
    }

    pub async fn read(&self, buf: &mut [u8]) -> Result<usize> {
        if !self.writer.read().await.connected {
            return Unexpected!("read from unconnected stream");
        }
        Ok(self.reader.write().await.read(buf).await?)
    }

    pub async fn write(&self, buf: &[u8]) -> Result<()> {
        self.writer.write().await.write(buf).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime::{sleep, spawn};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn run_server(addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        println!("Server running on {}", addr);

        loop {
            let (mut socket, _) = listener.accept().await?;
            spawn(async move {
                let mut buf = vec![0; 1024];
                let response = b"ABCDEFG"; // 固定返回的7个字符

                loop {
                    let n = socket.read(&mut buf).await.unwrap();

                    if n == 0 {
                        break;
                    }

                    socket.write_all(response).await.unwrap();
                }
            });
        }
    }

    #[derive(Copy, Clone)]
    struct Command(u16);

    impl From<Command> for [u8; 3] {
        fn from(value: Command) -> Self {
            let bytes = value.0.to_le_bytes();
            [bytes[1], bytes[0], bytes[0].wrapping_add(bytes[1])]
        }
    }

    #[test::case]
    #[ignore]
    async fn test_stream_read() {
        test::config!(config::Root { ..default!() });
        spawn(async move { run_server("127.0.0.1:9999").await });
        sleep(Duration::from_secs(1)).await;

        // let stream = TcpStream::connect("192.168.9.111".parse()?, 50000)
        let stream = TcpStream::connect("127.0.0.1".parse()?, 9999).await?;

        let stream_for_spawn = stream.clone();
        spawn(async move {
            let mut buffer = vec![0; 1024];
            loop {
                let mut duration = Duration::from_millis(200);
                match stream_for_spawn.read(&mut buffer).await {
                    Ok(n) if n >= 7 => {
                        warn!("read {n}");
                        buffer.clear();
                        buffer.resize(1024, 0);
                    }
                    Ok(_) => { /* Do nothing for reads less than 7 bytes */ }
                    Err(e) => {
                        warn!("Failed to read from stream: {:?}", e);
                        duration = Duration::from_secs(5);
                    }
                }
                sleep(duration).await;
            }
        });

        const QUERY: Command = Command(0x810A);

        let cmd: [u8; 3] = QUERY.into();
        stream.write(cmd.as_slice()).await?;
        sleep(Duration::from_secs(3)).await;

        let cmd: [u8; 3] = QUERY.into();
        stream.write(cmd.as_slice()).await?;
        sleep(Duration::from_secs(3)).await;

        let cmd: [u8; 3] = QUERY.into();
        stream.write(cmd.as_slice()).await?;
        sleep(Duration::from_secs(3)).await;
    }
}
