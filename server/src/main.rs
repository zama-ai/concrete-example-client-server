use byteorder::ReadBytesExt;
use commons::{ArithmeticType, Ident};
use concrete::{set_server_key, FheUint16, FheUint3, ServerKey};
use serde::de::DeserializeOwned;
use std::net::{TcpListener, TcpStream};

use colored::*;

fn deserialize_and_compute_circuit<T>(mut stream: &mut TcpStream) -> Result<T, String>
where
    T: DeserializeOwned + ArithmeticType,
{
    let formula: String = bincode::deserialize_from(&mut stream).unwrap();
    println!(
        "{}{}",
        "[Server] <---- [Client]: Received clear formula: ".blue(),
        formula.blue()
    );
    let valued_idents: Vec<(Ident, T)> = bincode::deserialize_from(&mut stream).unwrap();
    println!(
        "{}",
        "[Server] <---- [Client]: Received encrypted values".blue()
    );

    println!("{}", "[Server] : Homomorphically evaluating circuit".blue());
    let begin = std::time::Instant::now();
    let result = commons::tokenize(&formula)
        .map_err(|err| format!("Error parsing formula: {}", err))
        .and_then(|tokens| {
            commons::to_postfix(tokens)
                .map_err(|err| format!("Error converting to postfix notation: {}", err))
        })
        .and_then(|postfix| {
            commons::execute(postfix, valued_idents).map_err(|err| format!("{}", err))
        });
    let end = std::time::Instant::now();
    println!(
        "{}{}{}",
        "[Server] : Evaluation took ".blue(),
        (end - begin).as_secs(),
        " seconds".blue()
    );
    result
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    println!(
        "{}",
        "[Server] <---- [Client]: Receiving server keys from client".blue()
    );
    {
        let server_keys: ServerKey = bincode::deserialize_from(&mut stream).unwrap();
        set_server_key(server_keys);
    }

    loop {
        println!(
            "{}",
            "[Server] <---- [Client]: Waiting for user choice".blue()
        );
        let choice = stream.read_u8()?;
        match choice {
            0 => {
                println!("{}", "[Server] <---- [Client]: User said good bye".blue());
                break;
            }
            1 => loop {
                println!(
                    "{}",
                    "[Server] <---- [Client]: User wants to evaluate FheUint3 circuit".blue()
                );
                let result = deserialize_and_compute_circuit::<FheUint3>(&mut stream);
                bincode::serialize_into(&mut stream, &result).unwrap();
                println!(
                    "{}",
                    "[Server] -----> [Client]: Sending encrypted result".blue()
                );
                if result.is_ok() {
                    break;
                }
            },
            2 => loop {
                println!(
                    "{}",
                    "[Server] <---- [Client]: User wants to evaluate FheUint16 circuit".blue()
                );
                let result = deserialize_and_compute_circuit::<FheUint16>(&mut stream);
                bincode::serialize_into(&mut stream, &result).unwrap();
                println!(
                    "{}",
                    "[Server] ----> [Client]: Sending encrypted result".blue()
                );
                if result.is_ok() {
                    break;
                }
            },
            _ => panic!("Internal error, user was allowed to chose non supported type"),
        }
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server is listening");

    // accept connections and process them serially
    for stream in listener.incoming() {
        println!("A client initiated connection");
        std::thread::spawn(move || handle_client(stream?));
    }
    Ok(())
}
