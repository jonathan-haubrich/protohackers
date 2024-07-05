use std::{
    io::{Read, Write},
    net::TcpListener,
    thread::JoinHandle
};

fn main() {
    let mut connections: Vec<JoinHandle<()>> = Vec::new();
    let server = TcpListener::bind("0.0.0.0:8144");

    if let Ok(server) = server {
        for connection in server.incoming() {
            if let Ok(mut stream) = connection {
                let thread = std::thread::spawn(move || {
                    println!("Received connection from: {:?}", stream.peer_addr());
                    let mut read_buf = vec![0u8; 4096];
                    while let Ok(bytes_read) = stream.read(&mut read_buf) {
                        if 0 == bytes_read {
                            println!("Connection closed by remote end");
                            break;
                        }

                        println!("Bytes read from read: {:?}", bytes_read);
                        if let Err(error) =  stream.write(&read_buf[0..bytes_read]) {
                            println!("Failed to write to remote: {:?}", error);
                            break;
                        }
                    }
                });
                connections.push(thread);
            }
        }
    }

    connections.into_iter().for_each(|c| { c.join().unwrap() });
}
