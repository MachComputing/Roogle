use protocol::nonblocking::master;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() {
    console_subscriber::init();

    let args: Vec<String> = std::env::args().collect();
    let file = File::open(&args[1]).await.unwrap();
    let mut file = BufReader::new(file);
    let master = master::Master::new().await;

    let (inlet_write, inlet_read) = async_channel::bounded::<Box<[u8]>>(100);

    let master_thread = tokio::spawn(async move { master.dispatch(inlet_read).await });

    loop {
        let mut buf = String::new();
        let n = file.read_line(&mut buf).await;
        match n {
            Ok(0) => {
                break;
            }
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error reading file: {}", err);
                break;
            }
        };

        inlet_write.send(Box::from(buf.into_bytes())).await.unwrap();
    }
    inlet_write.close();
    master_thread.await.unwrap();
}
