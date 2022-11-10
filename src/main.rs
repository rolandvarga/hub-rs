use std::{
    fs,
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    thread,
    time::Duration,
    usize,
};

use hub_rs::{Message, MsgType, WorkerQueue};

const MAX_WORKERS: usize = 4;
const BUFFER_SIZE: usize = 50;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = WorkerQueue::new(MAX_WORKERS);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // pool.execute(|| {
                //     handle_connection(stream);
                // });
                thread::spawn(move || {
                    // handle_client(stream);
                    handle_stream(stream);
                });
            }
            Err(e) => {
                error!("Error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            // echo everything!
            stream.write(&data[0..size]).unwrap();
            true
        }
        Err(_) => {
            println!(
                "An error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn handle_stream(mut stream: TcpStream) {
    // TODO check for any expired connections
    info!("connected client: {}", stream.peer_addr().unwrap());

    let mut data = [0; BUFFER_SIZE]; // 50 bytes
    let size = stream.read(&mut data).unwrap();

    while match stream.read(&mut data) {
        Ok(size) => {
            stream
                .write(format!("received {} bytes; processing\n", size).as_bytes())
                .unwrap();

            let msg = Message::deserialize(&data);

            match &msg.msg_type {
                MsgType::Identify => {
                    stream
                        .write(format!("{}\n", stream.peer_addr().unwrap()).as_bytes())
                        .unwrap();
                }
                MsgType::List => {
                    info!("LIST");
                }
                MsgType::Relay => {
                    info!("RELAY -> {}", msg.body);
                }
                _ => {
                    info!("unknown message type");
                }
            }

            stream.write_all(b"request done\n").unwrap();
            stream.flush().unwrap();

            true
        }
        Err(_) => {
            println!(
                "An error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}
