use std::io::{self, BufReader, Error, ErrorKind, Read, Write};

use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A request message that can be sent to the proxy service.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Request {
    Transform {
        task: Value,
        args: Value,
        inputs: Vec<String>,
    },
}

/// A reply message from the proxy service.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Reply {
    Transform(Result<Value, String>),
}

macro_rules! io_impl {
    ($struct:ty) => {
        impl $struct {
            /// Serialize the message
            pub fn encode(&self) -> io::Result<Vec<u8>> {
                let mut buf = Vec::new();
                self.serialize(&mut Serializer::new(&mut buf))
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
                Ok(buf)
            }

            /// Deserialize the message
            pub fn decode(bytes: &[u8]) -> io::Result<Self> {
                rmp_serde::from_slice(&bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e))
            }

            /// Send the message on to a writer
            pub fn send<W: Write>(&self, writer: &mut W) -> io::Result<()> {
                let bytes = self.encode()?;
                let size = bytes.len().to_ne_bytes();
                let _ = writer.write(&size)?;
                let _ = writer.write(&bytes)?;
                writer.flush()?;
                Ok(())
            }

            /// Receive a message from a reader
            pub fn recv<R: Read>(reader: &mut R) -> io::Result<Self> {
                let mut reader = BufReader::new(reader);
                let mut size = [0u8; std::mem::size_of::<usize>()];
                let _ = reader.read(&mut size)?;
                let size = usize::from_ne_bytes(size);
                let mut bytes = vec![0u8; size];
                let _ = reader.read(&mut bytes)?;
                Self::decode(&bytes)
            }
        }
    };
}

io_impl!(Request);
io_impl!(Reply);

#[cfg(test)]
mod tests {
    use std::io::{Seek, SeekFrom};

    use serde_json::json;

    use super::*;

    #[test]
    fn send_request_recv_reply() {
        let mut file = tempfile::tempfile().unwrap();
        let request = Request::Transform {
            task: json!({ "task": "" }),
            args: json!({ "args": "" }),
            inputs: Vec::new(),
        };

        request.send(&mut file).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        let got = Request::recv(&mut file).unwrap();

        assert_eq!(request, got);
    }

    #[test]
    fn send_reply_recv_reply() {
        let mut file = tempfile::tempfile().unwrap();
        let reply = Reply::Transform(Ok(json!({ "success": true })));

        reply.send(&mut file).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        let got = Reply::recv(&mut file).unwrap();

        assert_eq!(reply, got);
    }
}
