use crate::nonblocking::protocol::Protocol;
use crate::Payload;
use std::collections::HashMap;
use tokio::io::{BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

pub struct Mapper {
    pub recv_master_socket: BufReader<OwnedReadHalf>,
    pub send_master_socket: BufWriter<OwnedWriteHalf>,
}

impl Mapper {
    pub fn new(stream: TcpStream) -> Self {
        let (recv, send) = stream.into_split();
        let recv_master_socket = BufReader::new(recv);
        let send_master_socket = BufWriter::new(send);

        Self {
            recv_master_socket,
            send_master_socket,
        }
    }

    pub async fn start_listening(&mut self) {
        loop {
            let mut protocol = Protocol::new();
            let payload = protocol.recv_msg(&mut self.recv_master_socket).await;

            match payload {
                Some(Payload::Work { block }) => {
                    // TODO: Mapper
                    let block = String::from_utf8(block.to_vec()).unwrap();
                    let mut tokens: HashMap<String, u32> = HashMap::new();
                    for word in block.split(|c: char| !c.is_alphanumeric()) {
                        let word = word.to_lowercase();
                        let count = tokens.entry(word).or_insert(0);
                        *count += 1;
                    }
                    let payload = Payload::WorkOk { tokens };
                    protocol
                        .send_msg(&mut self.send_master_socket, payload)
                        .await;
                }
                _ => {
                    return;
                }
            }
        }
    }
}