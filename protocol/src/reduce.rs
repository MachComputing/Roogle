use crate::{Payload, Protocol};
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;

pub struct Reducer {
    pub recv_master_socket: BufReader<TcpStream>,
    pub send_master_socket: BufWriter<TcpStream>,
}

impl Reducer {
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
                Some(Payload::Reduce { tokens }) => {
                    // TODO: Reducer
                    println!("{:?}", tokens);
                    protocol.send_msg(&mut self.send_master_socket, Payload::ReduceOk);
                }
                _ => {
                    return;
                }
            }
        }
    }
}
