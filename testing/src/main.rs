use std::io::BufRead;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n_reducers = args[1].parse::<usize>().unwrap();
    let n_mappers = args[2].parse::<usize>().unwrap();

    let bench_file = "testinputLarger.txt";
    let mut line_count = 0;
    let file = std::fs::File::open(bench_file).unwrap();
    let reader = std::io::BufReader::new(file);
    for _ in reader.lines() {
        line_count += 1;
    }

    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .spawn()
        .expect("failed to execute process")
        .wait()
        .unwrap();

    let mut master = Command::new("target/release/master")
        .arg("testinputLarger.txt")
        .spawn()
        .expect("failed to execute process");
    println!("Waiting for master to start");
    thread::sleep(Duration::from_secs(2));

    let start = Instant::now();
    let mappers: Vec<Child> = (0..n_mappers)
        .map(|i| {
            Command::new("target/release/mapper")
                .arg("127.0.0.1:8000")
                .spawn()
                .expect("failed to execute process")
        })
        .collect();

    let reducers: Vec<Child> = (0..n_reducers)
        .map(|i| {
            Command::new("target/release/reducer")
                .arg("127.0.0.1:8001")
                .spawn()
                .expect("failed to execute process")
        })
        .collect();
    master.wait().unwrap();
    let elapsed = start.elapsed();
    println!("Time elapsed: {:?}", elapsed);
    println!(
        "Lines per second is {}",
        line_count as f64 / elapsed.as_secs_f64()
    );
}
