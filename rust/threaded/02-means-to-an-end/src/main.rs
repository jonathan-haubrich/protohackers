use std::{
    collections::HashMap, io::{Cursor, Read, Write}, net::TcpListener, thread::JoinHandle
};

use binrw::{
    binrw,
    BinRead,
    BinWriterExt
};

#[derive(Debug)]
#[binrw]
struct InsertMessage {
    pub r#type: u8,
    pub timestamp: i32,
    pub price: i32
}

#[derive(Debug)]
#[binrw]
struct QueryMessage {
    pub r#type: u8,
    pub mintime: i32,
    pub maxtime: i32
}

#[derive(Debug, PartialEq)]
#[binrw]
struct Response {
    pub average: i32
}

fn calc_average(message: &QueryMessage, prices: &HashMap<i32, i32>) -> i32 {
    let mut sum = 0i64;
    let mut count = 0;

    for (timestamp, price) in prices.iter() {
        if message.mintime <= *timestamp && *timestamp <= message.maxtime {
            sum += *price as i64;
            count += 1;
        }
    }

    if count == 0 {
        return 0;
    }

    (sum / count) as i32
}

fn handle_message(message: &[u8], prices: &mut HashMap<i32, i32>) -> Option<Response> {
    if message[0].is_ascii() {
        match char::from(message[0]) {
            'I' => {
                if let Ok(message) = InsertMessage::read_be(&mut Cursor::new(&message)) {
                    prices.insert(message.timestamp, message.price);
                }
            },
            'Q' => {
                if let Ok(message) = QueryMessage::read_be(&mut Cursor::new(&message)) {
                    return Some(Response {
                        average: calc_average(&message, prices)
                    });
                }
            },
            _ => { }
        }
    }
    
    None
}


fn main() {
    let mut connections: Vec<JoinHandle<()>> = Vec::new();
    let server = TcpListener::bind("0.0.0.0:8144");

    if let Ok(server) = server {
        for connection in server.incoming() {
            if let Ok(mut stream) = connection {
                let thread = std::thread::spawn(move || {
                    println!("Received connection from: {:?}", stream.peer_addr());
                    let mut prices: HashMap<i32, i32> = HashMap::new();
                    let mut buf = vec![0u8; 9];
                    while let Ok(_) = stream.read_exact(&mut buf) {
                        println!("Received from remote: {:?}", buf);

                        if let Some(response) = handle_message(&buf, &mut prices) {
                            let mut buf = vec![0u8; 4];
                            let mut writer = Cursor::new(&mut buf);
                            writer.write_be(&response).unwrap();
                            if let Err(error) = stream.write_all(&writer.into_inner()) {
                                println!("Failed to write to remote: {:?}", error);
                                break;
                            }
                        }

                    }
                    println!("Connection closed");
                });
                connections.push(thread);
            }
        }
    }

    connections.into_iter().for_each(|c| { c.join().unwrap() });
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_average() {
        let prices: HashMap<i32, i32> = [
            (1, 10),
            (2, 20),
            (3, 30),
            (4, 40),
            (5, 50),
        ]
        .iter()
        .cloned()
        .collect();

        let message = QueryMessage {
            r#type: 0,
            mintime: 2,
            maxtime: 4,
        };

        let average = calc_average(&message, &prices);
        assert_eq!(average, 30);
    }

    #[test]
    fn test_handle_message_insert() {
        let mut prices: HashMap<i32, i32> = HashMap::new();
        let message = [b'I', 0, 0, 0, 1, 0, 0, 0, 10];

        handle_message(&message, &mut prices);

        assert_eq!(prices.get(&1), Some(&10));
    }

    #[test]
    fn test_handle_message_query() {
        let mut prices: HashMap<i32, i32> = [
            (1, 10),
            (2, 20),
            (3, 30),
            (4, 40),
            (5, 50),
        ]
        .iter()
        .cloned()
        .collect();

        let message = [b'Q', 0, 0, 0, 2, 0, 0, 0, 4];

        let response = handle_message(&message, &mut prices);

        assert_eq!(response, Some(Response { average: 30 }));
    }
}
