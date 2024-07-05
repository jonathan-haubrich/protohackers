use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    thread::JoinHandle,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct PrimeMessage {
    pub method: String,
    pub number: serde_json::Number
}

#[derive(Serialize)]
struct PrimeResponse {
    pub method: String,
    pub prime: bool
}

fn is_prime(message: &PrimeMessage) -> bool {
    let number = if message.number.is_f64() {
        if message.number.as_f64().unwrap().fract() == 0.0 {
            Some(message.number.as_f64().unwrap() as u64)
        } else {
            None
        }
    } else if message.number.is_i64() {
        if message.number.as_i64().unwrap() >= 0 {
            Some(message.number.as_i64().unwrap() as u64)
        } else {
            None
        }
    } else {
        Some(message.number.as_u64().unwrap())
    };
    
    if let Some(number) = number {
        return num_prime::nt_funcs::is_prime(&number, None).probably();
    } else {
        return false;
    }
}

fn handle_line(buf: &String) -> String {
    if let Ok(message) = serde_json::from_str::<PrimeMessage>(&buf) {
        if "isPrime" != message.method {
            return String::from("{}");
        }

        return serde_json::to_string(&PrimeResponse {
            method: String::from("isPrime"),
            prime: is_prime(&message)
        }).unwrap();
    } else {
        return String::from("{}");
    }
}

fn main() {
    let mut connections: Vec<JoinHandle<()>> = Vec::new();
    let server = TcpListener::bind("0.0.0.0:8144");

    if let Ok(server) = server {
        for connection in server.incoming() {
            if let Ok(mut stream) = connection {
                let thread = std::thread::spawn(move || {
                    println!("Received connection from: {:?}", stream.peer_addr());
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut buf = String::new();
                    while let Ok(bytes_read) = reader.read_line(&mut buf) {
                        if 0 == bytes_read {
                            println!("Connection closed by remote end");
                            break;
                        }

                        println!("Bytes read from read: {:?}", bytes_read);
                        println!("Received: {:?}", buf);

                        let response = handle_line(&buf);
                        println!("Responding with: {:?}", response);
                        
                        if let Err(error) = stream.write(response.as_bytes()) {
                            println!("Failed to write to remote: {:?}", error);
                            break;
                        }
                        if let Err(error) = stream.write(b"\n") {
                            println!("Failed to write to remote: {:?}", error);
                            break;
                        }

                        buf.clear();
                    }
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
    fn test_handle_line_is_prime_f64_with_fraction() {
        let input = r#"{"method": "isPrime", "number": 17.5}"#;
        let expected_output = r#"{"method":"isPrime","prime":false}"#;
        assert_eq!(handle_line(&input.to_string()), expected_output);
    }

    #[test]
    fn test_handle_line_is_not_prime_f64_with_fraction() {
        let input = r#"{"method": "isPrime", "number": 10.5}"#;
        let expected_output = r#"{"method":"isPrime","prime":false}"#;
        assert_eq!(handle_line(&input.to_string()), expected_output);
    }

    #[test]
    fn test_handle_line_is_prime_f64() {
        let input = r#"{"method": "isPrime", "number": 17.0}"#;
        let expected_output = r#"{"method":"isPrime","prime":true}"#;
        assert_eq!(handle_line(&input.to_string()), expected_output);
    }

    #[test]
    fn test_handle_line_is_prime_i64() {
        let input = r#"{"method": "isPrime", "number": 17}"#;
        let expected_output = r#"{"method":"isPrime","prime":true}"#;
        assert_eq!(handle_line(&input.to_string()), expected_output);
    }

    #[test]
    fn test_handle_line_is_not_prime_f64() {
        let input = r#"{"method": "isPrime", "number": 10.0}"#;
        let expected_output = r#"{"method":"isPrime","prime":false}"#;
        assert_eq!(handle_line(&input.to_string()), expected_output);
    }

    #[test]
    fn test_handle_line_is_not_prime_i64() {
        let input = r#"{"method": "isPrime", "number": 10}"#;
        let expected_output = r#"{"method":"isPrime","prime":false}"#;
        assert_eq!(handle_line(&input.to_string()), expected_output);
    }

    #[test]
    fn test_handle_line_invalid_method() {
        let input = r#"{"method": "invalidMethod", "number": 5}"#;
        let expected_output = "{}";
        assert_eq!(handle_line(&input.to_string()), expected_output);
    }

    #[test]
    fn test_handle_line_invalid_number() {
        let input = r#"{"method": "isPrime", "number": "abc"}"#;
        let expected_output = "{}";
        assert_eq!(handle_line(&input.to_string()), expected_output);
    }

    #[test]
    fn test_handle_line_is_prime_negative_i64() {
        let input = r#"{"method": "isPrime", "number": -17}"#;
        let expected_output = r#"{"method":"isPrime","prime":false}"#;
        assert_eq!(handle_line(&input.to_string()), expected_output);
    }

    #[test]
    fn test_handle_line_is_prime_negative_f64() {
        let input = r#"{"method": "isPrime", "number": -17.0}"#;
        let expected_output = r#"{"method":"isPrime","prime":false}"#;
        assert_eq!(handle_line(&input.to_string()), expected_output);
    }
}
