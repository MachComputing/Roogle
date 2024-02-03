mod xmlhelper;

use protocol::nonblocking::master;
use std::fs::File;
use std::io::BufReader;
use xml::reader::XmlEvent;
use xml::EventReader;

#[tokio::main]
async fn main() {
    console_subscriber::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {name} <bench_file>", name = args[1]);
        std::process::exit(1);
    }

    let file = File::open(&args[1]).unwrap();
    let file = BufReader::new(file);
    let mut event_reader = EventReader::new(file);

    let master = master::Master::new().await;
    println!("Master started");

    let (inlet_write, inlet_read) = async_channel::bounded::<String>(100);

    let master_thread = tokio::spawn(async move { master.dispatch(inlet_read).await });
    let mut n_lines = 0;

    loop {
        print!("{} lines processed\r", n_lines);
        match event_reader.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                if name.local_name == "page" {
                    match xmlhelper::recursive_ev_find(&mut event_reader, &["revision", "text"]) {
                        None => {
                            println!("Error: No text found");
                            break;
                        }
                        Some(text) => {
                            inlet_write.send(text).await.unwrap();
                            n_lines += 1;
                        }
                    };
                }
            }
            _ => {}
        }
    }

    inlet_write.close();
    master_thread.await.unwrap();
}
