use crate::nonblocking::protocol::Protocol;
use crate::Payload;
use async_channel::{Receiver, Sender};
use std::time::Duration;
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio::{select, time};

pub struct Master {
    map_socket: TcpListener,
    reduce_socket: TcpListener,
}

impl Master {
    pub async fn new() -> Self {
        Self {
            map_socket: TcpListener::bind("0.0.0.0:8000").await.unwrap(),
            reduce_socket: TcpListener::bind("0.0.0.0:8001").await.unwrap(),
        }
    }

    pub async fn dispatch(&self, inlet: Receiver<Box<[u8]>>) {
        let (outlet_write, outlet_read) = async_channel::bounded::<Payload>(100);
        let mut map_handles = vec![];
        let mut red_handles = vec![];

        let sleep = time::sleep(Duration::from_millis(100));
        tokio::pin!(sleep);
        loop {
            if map_handles.len() > 0 && map_handles.iter().all(|h: &JoinHandle<()>| h.is_finished())
            {
                outlet_write.close();
            }

            if red_handles.len() > 0 && red_handles.iter().all(|h: &JoinHandle<()>| h.is_finished())
            {
                break;
            }

            let (connection, listener_type) = select! {
                mapper = self.map_socket.accept() => (mapper, "mapper"),
                reducer = self.reduce_socket.accept() => (reducer, "reducer"),
                timer = &mut sleep => continue,
            };

            let (stream, host) = connection.unwrap();
            // println!("New connection at {} from a {}", host, listener_type);

            if listener_type == "mapper" {
                let inlet = Receiver::clone(&inlet);
                let outlet_write = Sender::clone(&outlet_write);
                map_handles.push(tokio::spawn(async move {
                    dispatch_mapper(stream, inlet, outlet_write).await
                }));
            } else {
                let outlet_read = Receiver::clone(&outlet_read);
                red_handles.push(tokio::spawn(async move {
                    dispatch_reducer(stream, outlet_read).await
                }));
            }
        }
    }
}

async fn dispatch_mapper(
    mut stream: TcpStream,
    inlet: Receiver<Box<[u8]>>,
    outlet: Sender<Payload>,
) {
    let (recv_socket, send_socket) = stream.into_split();

    let sender = tokio::spawn(async move {
        let mut protocol = Protocol::new();
        let mut send_buf = BufWriter::new(send_socket);
        loop {
            let unit = inlet.recv().await;
            match unit {
                Ok(block) => {
                    protocol
                        .send_msg(&mut send_buf, Payload::Work { block })
                        .await;
                }
                Err(err) => {
                    break;
                }
            }
        }
    });

    let receiver = tokio::spawn(async move {
        let mut protocol = Protocol::new();
        let mut recv_buf = BufReader::new(recv_socket);
        let max_tries = 10usize;
        let mut tries = max_tries;

        let mut work_oks = 0usize;
        loop {
            if tries == 0 {
                break;
            }

            let payload =
                timeout(Duration::from_millis(100), protocol.recv_msg(&mut recv_buf)).await;
            if let Err(_) = payload {
                tries -= 1;
                continue;
            }
            tries = max_tries;

            match payload.unwrap() {
                Some(p @ Payload::WorkOk { .. }) => {
                    outlet.send(p).await.unwrap();
                    work_oks += 1;
                }
                Some(p) => {
                    eprintln!("Error: Unexpected payload: {:?}", p);
                    continue;
                }
                None => {
                    break;
                }
            }
        }
        // println!("Received {} work_ok", work_oks);
    });

    sender.await.unwrap();
    receiver.await.unwrap();
}

async fn dispatch_reducer(mut stream: TcpStream, inlet: Receiver<Payload>) {
    let (recv_socket, send_socket) = stream.into_split();

    let sender = tokio::spawn(async move {
        let mut protocol = Protocol::new();
        let mut send_buf = BufWriter::new(send_socket);
        loop {
            let unit = inlet.recv().await;
            match unit {
                Ok(Payload::WorkOk { tokens }) => {
                    protocol
                        .send_msg(&mut send_buf, Payload::Reduce { tokens })
                        .await;
                }
                Ok(_) => {
                    eprintln!("Error: Unexpected payload")
                }
                _ => {
                    break;
                }
            }
        }
    });

    let receiver = tokio::spawn(async move {
        let mut recv_buf = BufReader::new(recv_socket);
        let mut protocol = Protocol::new();
        let mut recv_count = 0usize;
        let max_tries = 10usize;
        let mut tries = max_tries;
        loop {
            let payload =
                timeout(Duration::from_millis(100), protocol.recv_msg(&mut recv_buf)).await;
            if let Err(_) = payload {
                tries -= 1;
                continue;
            }
            tries = max_tries;

            match payload.unwrap() {
                Some(p @ Payload::ReduceOk) => {
                    recv_count += 1;
                }
                Some(p) => {
                    eprintln!("Error: Unexpected payload: {:?}", p);
                    continue;
                }
                None => {
                    break;
                }
            }
        }
        // println!("Received {} reduce_ok", recv_count);
    });

    sender.await.unwrap();
    receiver.await.unwrap();
}
