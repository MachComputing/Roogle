pub mod nonblocking;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;

pub struct Protocol {
    message_id: usize,
}

impl Protocol {
    pub fn new() -> Self {
        Protocol { message_id: 0 }
    }

    pub fn send_msg(&mut self, to: &mut BufWriter<TcpStream>, payload: Payload) {
        let msg = Message {
            id: self.message_id,
            payload,
        };
        serde_json::to_writer(&mut *to, &msg).unwrap();
        to.write(b"\0")
            .expect("Error: Could not communicate with socket");
        self.message_id += 1;
        to.flush().unwrap();
    }

    pub fn recv_msg<ST>(&mut self, from: &mut BufReader<ST>) -> Option<Payload>
    where
        ST: Read + Write,
    {
        let mut res: Vec<u8> = vec![];
        match from.read_until(b'\0', &mut res) {
            Ok(0) => None,
            Ok(_) => {
                let msg: Result<Message, _> = serde_json::from_slice(&res[..res.len() - 1]);
                if let Ok(msg) = msg {
                    Some(msg.payload)
                } else {
                    println!("Error: Could not parse message: {:?}", res);
                    None
                }
            }
            Err(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "msg_id")]
    pub id: usize,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Reduce { tokens: HashMap<String, u32> },
    ReduceOk,

    Work { block: String },
    WorkOk { tokens: HashMap<String, u32> },

    DoneMap,
    DoneMapOk,
    DoneReduce,
    DoneReduceOk,
}
