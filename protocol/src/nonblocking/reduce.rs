use crate::nonblocking::protocol::Protocol;
use crate::Payload;
use tokio::io::{BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

pub struct Reducer {
    pub recv_master_socket: BufReader<OwnedReadHalf>,
    pub send_master_socket: BufWriter<OwnedWriteHalf>,
}

impl Reducer {
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
                Some(Payload::Reduce { tokens }) => {
                    // TODO: Reducer
                    // println!("{:?}", tokens);
                    protocol
                        .send_msg(&mut self.send_master_socket, Payload::ReduceOk)
                        .await;
                }
                _ => {
                    return;
                }
            }
        }
    }
}
