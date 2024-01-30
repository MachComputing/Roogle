use protocol::nonblocking::reduce::Reducer;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let stream = TcpStream::connect(&args[1]).await.unwrap();
    let mut runner = Reducer::new(stream);
    runner.start_listening().await;
}
