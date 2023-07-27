use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const REQUEST_SIZE: usize = std::mem::size_of::<Request>();
pub const REPLY_SIZE: usize = std::mem::size_of::<Reply>();

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

macro_rules! encode_decode {
    ($struct:ty) => {
        impl $struct {
            /// Serialize the message
            pub fn encode(&self) -> Result<Vec<u8>, String> {
                let mut buf = Vec::new();
                self.serialize(&mut Serializer::new(&mut buf))
                    .map_err(|e| e.to_string())?;
                Ok(buf)
            }

            /// Deserialize the message
            pub fn decode(bytes: &[u8]) -> Result<Self, String> {
                rmp_serde::from_slice(&bytes).map_err(|e| e.to_string())
            }
        }
    };
}

encode_decode!(Request);
encode_decode!(Reply);
