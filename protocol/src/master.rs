use crate::{Payload, Protocol};
use crossbeam::deque::{Injector, Stealer, Worker};
use std::io::{BufReader, BufWriter};
use std::net::TcpListener;
use std::sync::{Arc, RwLock};
use std::{iter, thread};

pub struct Master {
    map_socket: TcpListener,
    reduce_socket: TcpListener,
}

impl Master {
    pub fn new() -> Self {
        Self {
            map_socket: TcpListener::bind("0.0.0.0:8000").unwrap(),
            reduce_socket: TcpListener::bind("0.0.0.0:8001").unwrap(),
        }
    }

    pub fn start_listening_map(
        &self,
        map_global_queue: &Injector<String>,
        red_global_queue: &Injector<Payload>,
    ) {
        thread::scope(|scope| {
            let all_works_stealer = Arc::new(RwLock::new(vec![]));

            for stream in self.map_socket.incoming() {
                println!("New connection at MAP");
                let stream = stream.unwrap();
                let mut protocol = Protocol::new();

                let worker_queue: Worker<String> = Worker::new_fifo();
                let worker_stealer = worker_queue.stealer();
                all_works_stealer.write().unwrap().push(worker_stealer);
                let all_works_stealer = Arc::clone(&all_works_stealer);

                scope.spawn(move || {
                    let mut sender = BufWriter::new(stream.try_clone().unwrap());
                    let mut receiver = BufReader::new(stream);

                    loop {
                        let unit = find_task(
                            &worker_queue,
                            map_global_queue,
                            &all_works_stealer.read().unwrap(),
                        );
                        if unit.is_none() {
                            continue;
                        }

                        let unit = unit.unwrap();
                        let payload = Payload::Work {
                            block: Box::from(unit.as_bytes()),
                        };

                        protocol.send_msg(&mut sender, payload);
                        let payload = protocol.recv_msg(&mut receiver);

                        match payload {
                            Some(p @ Payload::WorkOk { .. }) => {
                                red_global_queue.push(p);
                            }
                            _ => {
                                println!("Error: {:?}", payload);
                                worker_queue.push(unit);
                            }
                        }
                    }
                });
            }
        })
    }

    pub fn start_listening_reduce(&self, red_global_queue: &Injector<Payload>) {
        thread::scope(|scope| {
            let all_works_stealer = Arc::new(RwLock::new(vec![]));

            for stream in self.reduce_socket.incoming() {
                println!("New connection at REDUCE");
                let stream = stream.unwrap();
                let mut protocol = Protocol::new();

                let worker_queue: Worker<Payload> = Worker::new_fifo();
                let worker_stealer = worker_queue.stealer();
                all_works_stealer.write().unwrap().push(worker_stealer);
                let all_works_stealer = Arc::clone(&all_works_stealer);

                scope.spawn(move || {
                    let mut sender = BufWriter::new(stream.try_clone().unwrap());
                    let mut receiver = BufReader::new(stream);

                    loop {
                        let unit = find_task(
                            &worker_queue,
                            red_global_queue,
                            &all_works_stealer.read().unwrap(),
                        );
                        if unit.is_none() {
                            continue;
                        }

                        let unit = unit.unwrap();
                        let tokens = match unit {
                            Payload::WorkOk { tokens } => tokens,
                            _ => {
                                println!("Error: Wrong unit in reducers {:?}", unit);
                                continue;
                            }
                        };

                        let payload = Payload::Reduce { tokens };

                        protocol.send_msg(&mut sender, payload);
                        let recv = protocol.recv_msg(&mut receiver);
                        match recv {
                            Some(Payload::ReduceOk) => {}
                            _ => {
                                println!("Error: {:?}", recv);
                            }
                        }
                    }
                });
            }
        })
    }
}

fn find_task<T>(local: &Worker<T>, global: &Injector<T>, stealers: &Vec<Stealer<T>>) -> Option<T> {
    // Pop a task from the local queue, if not empty.
    local.pop().or_else(|| {
        // Otherwise, we need to look for a task elsewhere.
        iter::repeat_with(|| {
            // Try stealing a batch of tasks from the global queue.
            global
                .steal_batch_and_pop(local)
                // Or try stealing a task from one of the other threads.
                .or_else(|| stealers.iter().map(|s| s.steal()).collect())
        })
        // Loop while no task was stolen and any steal operation needs to be retried.
        .find(|s| !s.is_retry())
        // Extract the stolen task, if there is one.
        .and_then(|s| s.success())
    })
}
