use protocol::mapper;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let stream = std::net::TcpStream::connect(&args[1]).unwrap();
    let mut runner = mapper::Mapper::new(
        std::io::BufReader::new(stream.try_clone().unwrap()),
        std::io::BufWriter::new(stream),
    );
    runner.start_listening();
}
