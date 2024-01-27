use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n_reducers = args[1].parse::<usize>().unwrap();
    let n_mappers = args[2].parse::<usize>().unwrap();

    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .spawn()
        .expect("failed to execute process")
        .wait()
        .unwrap();

    let mut master = Command::new("target/release/master")
        .arg("testinput.txt")
        .spawn()
        .expect("failed to execute process");

    let start = Instant::now();
    let mappers: Vec<Child> = (0..n_mappers)
        .map(|i| {
            Command::new("target/release/mapper")
                .stdout(Stdio::null())
                .arg("127.0.0.1:8000")
                .spawn()
                .expect("failed to execute process")
        })
        .collect();

    let reducers: Vec<Child> = (0..n_reducers)
        .map(|i| {
            Command::new("target/release/reducer")
                .stdout(Stdio::null())
                .arg("127.0.0.1:8001")
                .spawn()
                .expect("failed to execute process")
        })
        .collect();
    master.wait().unwrap();
    println!("Time elapsed: {:?}", start.elapsed());
}
