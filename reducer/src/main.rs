use protocol::nonblocking::reduce::Reducer;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {name} <master-ip>", name = args[1]);
        std::process::exit(1);
    }

    let stream = TcpStream::connect(&args[1]).await.unwrap();
    let mut runner = Reducer::new(stream);
    runner.start_listening().await;
}
