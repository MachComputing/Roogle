use crate::nonblocking::protocol::Protocol;
use crate::Payload;
use std::collections::HashMap;
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
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
                    let block = block
                        .iter()
                        .map(|byte| *byte as char)
                        .collect::<Vec<char>>();

                    let mut tokens: HashMap<String, u32> = HashMap::new();
                    let lexer = nlp::lexer::Lexer::new(&block);

                    for word in lexer.into_iter() {
                        let word = word.to_lowercase();
                        let count = tokens.entry(word).or_insert(0);
                        *count += 1;
                    }

                    let payload = Payload::WorkOk { tokens };
                    protocol
                        .send_msg(&mut self.send_master_socket, payload)
                        .await;
                }
                Some(Payload::DoneMap) => {
                    protocol
                        .send_msg(&mut self.send_master_socket, Payload::DoneMapOk)
                        .await;

                    self.send_master_socket
                        .flush()
                        .await
                        .expect("Error trying to flush socket");
                    break;
                }
                Some(p) => {
                    eprintln!("Error: Unexpected payload: {:?}", p);
                    continue;
                }
                _ => {
                    break;
                }
            }
        }
    }
}
