use std::io::BufRead;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!(
            "Usage: {name} <master|<master-ip>> <n_reducers> <n_mappers> <bench_file>",
            name = args[1]
        );
        std::process::exit(1);
    }

    let spawn_master = args[1] == "master";
    let mut master_ip = "127.0.0.1";
    if !spawn_master {
        master_ip = &*args[1];
    }

    let n_reducers = args[2].parse::<usize>().unwrap();
    let n_mappers = args[3].parse::<usize>().unwrap();

    let bench_file = args[4].clone();

    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .spawn()
        .expect("failed to execute process")
        .wait()
        .unwrap();

    let mut master: Option<Child> = None;
    if spawn_master {
        master = Some(
            Command::new("target/release/master")
                .arg(bench_file)
                .spawn()
                .expect("failed to execute process"),
        );
        println!("Waiting for master to start");
        thread::sleep(Duration::from_secs(5));
    }

    let start = Instant::now();
    let _mappers: Vec<Child> = (0..n_mappers)
        .map(|i| {
            thread::sleep(Duration::from_millis(100));
            Command::new("target/release/mapper")
                .arg(master_ip.to_owned() + ":8000")
                .spawn()
                .expect("failed to execute process")
        })
        .collect();

    let _reducers: Vec<Child> = (0..n_reducers)
        .map(|i| {
            thread::sleep(Duration::from_millis(100));
            Command::new("target/release/reducer")
                .arg(master_ip.to_owned() + ":8001")
                .spawn()
                .expect("failed to execute process")
        })
        .collect();

    if let Some(mut master) = master {
        master.wait().unwrap();
    }
    let elapsed = start.elapsed();
    println!("Time elapsed: {:?}", elapsed);
}
