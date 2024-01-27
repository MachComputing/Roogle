use crossbeam::deque::Injector;
use protocol::{master, Payload};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::thread;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let protocol = &master::Master::new();

    let map_global_queue: Injector<String> = Injector::new();
    let red_global_queue: Injector<Payload> = Injector::new();

    thread::scope(|scope| {
        // Start the mapping thread
        scope.spawn(|| {
            protocol.start_listening_map(&map_global_queue, &red_global_queue);
        });

        // Start the reducing thread
        scope.spawn(|| {
            protocol.start_listening_reduce(&red_global_queue);
        });

        // Read the file line by line and send the lines to the mapping thread
        if let Ok(lines) = read_lines(&args[1]) {
            for line in lines {
                if let Ok(l) = line {
                    map_global_queue.push(l);
                }
            }
        }
    })
}

// Function to read a file line by line
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
