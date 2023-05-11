//! Stream wrapper, which allows for custom interactions with the network protocol.
//! We might want to turn it into a fuzz testing framework for the network protocol.
use bytes::BytesMut;

use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::network_protocol::{Encoding, PeerMessage};
use crate::tcp;

pub struct Stream {
    stream: tcp::Stream,
    force_encoding: Option<Encoding>,
    protocol_buffers_supported: bool,
}

impl Stream {
    pub fn new(force_encoding: Option<Encoding>, stream: tcp::Stream) -> Self {
print_file_path_and_function_name!();

        Self { stream, force_encoding, protocol_buffers_supported: false }
    }

    fn encoding(&self) -> Option<Encoding> {
print_file_path_and_function_name!();

        if self.force_encoding.is_some() {
            return self.force_encoding;
        }
        if self.protocol_buffers_supported {
            return Some(Encoding::Proto);
        }
        return None;
    }

    pub async fn read(&mut self) -> Result<PeerMessage, std::io::Error> {
print_file_path_and_function_name!();

        'read: loop {
            let n = self.stream.stream.read_u32_le().await? as usize;
            let mut buf = BytesMut::new();
            buf.resize(n, 0);
            self.stream.stream.read_exact(&mut buf[..]).await?;
            for enc in [Encoding::Proto, Encoding::Borsh] {
                if let Ok(msg) = PeerMessage::deserialize(enc, &buf[..]) {
                    // If deserialize() succeeded but we expected different encoding, ignore the
                    // message.
                    if self.encoding().unwrap_or(enc) != enc {
                        println!("unexpected encoding, ignoring message");
                        continue 'read;
                    }
                    if enc == Encoding::Proto {
                        self.protocol_buffers_supported = true;
                    }
                    return Ok(msg);
                }
            }
            panic!("unknown encoding");
        }
    }

    pub async fn write(&mut self, msg: &PeerMessage) {
print_file_path_and_function_name!();

        if let Some(enc) = self.encoding() {
            self.write_encoded(&msg.serialize(enc)).await;
        } else {
            self.write_encoded(&msg.serialize(Encoding::Proto)).await;
            self.write_encoded(&msg.serialize(Encoding::Borsh)).await;
        }
    }

    async fn write_encoded(&mut self, msg: &[u8]) {
print_file_path_and_function_name!();

        self.stream.stream.write_u32_le(msg.len() as u32).await.unwrap();
        self.stream.stream.write_all(msg).await.unwrap();
        self.stream.stream.flush().await.unwrap();
    }
}
