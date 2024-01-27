use crate::{Payload, Protocol};
use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;

pub struct Mapper {
    pub recv_master_socket: BufReader<TcpStream>,
    pub send_master_socket: BufWriter<TcpStream>,
}

impl Mapper {
    pub fn new(
        recv_master_socket: BufReader<TcpStream>,
        send_master_socket: BufWriter<TcpStream>,
    ) -> Self {
        Self {
            recv_master_socket,
            send_master_socket,
        }
    }

    pub fn start_listening(&mut self) {
        loop {
            let mut protocol = Protocol::new();
            let payload = protocol.recv_msg(&mut self.recv_master_socket);
            println!("Received payload: {:?}", payload);
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
                    println!("{:?}", payload);
                    protocol.send_msg(&mut self.send_master_socket, payload);
                }
                _ => {
                    return;
                }
            }
        }
    }
}
