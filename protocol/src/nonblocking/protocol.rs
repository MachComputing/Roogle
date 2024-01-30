use crate::{Message, Payload};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};

pub struct Protocol {
    message_id: usize,
}

impl Protocol {
    pub fn new() -> Self {
        Protocol { message_id: 0 }
    }

    pub async fn send_msg<S>(&mut self, to: &mut BufWriter<S>, payload: Payload)
    where
        S: AsyncWrite + Unpin,
    {
        let msg = Message {
            id: self.message_id,
            payload,
        };
        let msg = serde_json::to_string(&msg).expect("Error: Could not serialize message");
        // Header: length of message
        let write_res = to.write_u32(msg.len() as u32).await;
        if let Err(_) = write_res {
            return;
        }

        // Body: message
        let write_res = to.write_all(msg.as_bytes()).await;
        if let Err(_) = write_res {
            return;
        }
        // to.flush().await.expect("Error trying to flush socket");
    }

    pub async fn recv_msg<S>(&mut self, from: &mut BufReader<S>) -> Option<Payload>
    where
        S: AsyncRead + Unpin,
    {
        let length = from.read_u32().await;
        if let Err(_) = length {
            return None;
        }

        let length = length.unwrap();
        let mut buf = vec![0; length as usize];
        let read = from.read_exact(&mut buf).await;
        if let Err(_) = read {
            return None;
        }

        let msg: Result<Message, _> = serde_json::from_slice(&buf);
        if let Ok(msg) = msg {
            Some(msg.payload)
        } else {
            None
        }
    }
}
