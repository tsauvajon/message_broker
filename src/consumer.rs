use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug)]
pub enum ConsumeError {
    ProtocolError,
    IoError(std::io::Error),
}

impl From<std::io::Error> for ConsumeError {
    fn from(err: std::io::Error) -> ConsumeError {
        ConsumeError::IoError(err)
    }
}

pub async fn consume_one(stream: &mut TcpStream) -> Result<Option<bytes::Bytes>, ConsumeError> {
    let (mut r, mut w) = stream.split();
    w.write_all(b"\0").await?;

    let mut buf = vec![0; 128];
    let mut value: Vec<u8> = vec![];
    loop {
        match r.read(&mut buf).await {
            Ok(0) => return Ok(None), // EOF
            Ok(n) => {
                if &buf[n - 1..n] == b"\0" {
                    value.append(&mut buf[..n - 1].to_vec());
                    break;
                }

                value.append(&mut buf[..n].to_vec());
            }
            Err(err) => return Err(ConsumeError::IoError(err)),
        };
    }

    let operand = &value[0];
    if *operand == b"+"[0] {
        value = value[1..].to_vec();
        Ok(Some(value.into()))
    } else if *operand == b"-"[0] {
        Ok(None)
    } else {
        Err(ConsumeError::ProtocolError)
    }
}
