#![allow(warnings)]

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{stdin, stdout, BufWriter};

use bincode;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::prelude::*;
use std::net::TcpStream;

use crate::details::{ask_for_exit, fhe16_from_stin, fhe3_from_stin, key_gen};
use concrete::prelude::*;
use concrete::{ClientKey, FheUint16, FheUint3};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use commons;
use commons::Ident;

mod details;

use colored::*;

fn ask_values_for_idents<T, F: Fn(&mut ClientKey) -> T>(
    unique_idents: Vec<Ident>,
    client_keys: &mut ClientKey,
    ask_fn: F,
) -> Vec<(Ident, T)> {
    let mut valued_idents = Vec::<(Ident, T)>::with_capacity(unique_idents.len());
    for ident in unique_idents {
        print!("Value for '{}', ", ident.as_str());
        let value = ask_fn(client_keys);
        valued_idents.push((ident, value));
    }

    valued_idents
}

// Would be great to get modulus from the FheType itself
fn inner_loop_logic<FheType, ClearType, F>(
    mut stream: &mut TcpStream,
    client_keys: &mut ClientKey,
    ask_fn: F,
    modulus: u32,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(&mut ClientKey) -> FheType,
    FheType: FheDecrypt<ClearType> + Serialize + DeserializeOwned,
    ClearType: Into<u32>,
{
    loop {
        print!("Enter a formula for the server to compute: ");
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;
        let tokens = match commons::tokenize(&input) {
            Err(e) => {
                println!("Error, invalid formula: {}", e);
                continue;
            }
            Ok(v) => v,
        };

        let unique_idents = commons::unique_idents(tokens.clone());
        let valued_idents = ask_values_for_idents(unique_idents, client_keys, &ask_fn);

        bincode::serialize_into(&mut stream, &input)?;
        println!(
            "{}",
            "[Client] ----> [Server]: Sending circuit in clear".blue()
        );
        bincode::serialize_into(&mut stream, &valued_idents)?;
        println!(
            "{}",
            "[Client] ----> [Server]: Sending encrypted values".blue()
        );

        let result: Result<FheType, String> = bincode::deserialize_from(&mut stream).unwrap();
        println!(
            "{}",
            "[Client] <---- [Server]: Receiving encrypted result".blue()
        );

        match result {
            Err(err) => {
                println!("Error: '{}'", err);
            }
            Ok(v) => {
                let clear_value_idents: Vec<(Ident, u32)> = valued_idents
                    .into_iter()
                    .map(|(ident, fhe_val)| {
                        let clear_val: ClearType = fhe_val.decrypt(client_keys);
                        (ident, clear_val.into())
                    })
                    .collect();

                let postfix = commons::to_postfix(tokens).unwrap();
                let clear_result = commons::execute(postfix, clear_value_idents).unwrap();

                let decrypted_result: ClearType = v.decrypt(client_keys);
                println!();
                println!("Server result is: {}", decrypted_result.into());
                println!("Expected result is: {}", clear_result % modulus);
                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut client_keys, mut server_keys) = key_gen()?;

    println!("{}", "[Client] ----> [Server]: Connecting to server".blue());
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;

    {
        println!(
            "{}",
            "[Client] ----> [Server]: Sending Bootstrap Keys to server".blue()
        );
        bincode::serialize_into(&mut stream, &server_keys)?;
        println!(
            "{}",
            "[Client] ----> [Server]: transaction successful".blue()
        );
    }

    loop {
        let choice = loop {
            println!();
            println!("Select the type to use for computations: ");
            println!("0: Disconnect");
            println!("1: FheUint3");
            println!("2: FheUint16");
            print!("Choice: ");
            stdout().flush()?;
            let mut line = String::new();
            stdin().read_line(&mut line)?;
            let choice = match line[..line.len() - 1].parse::<u8>() {
                Ok(v) => v,
                Err(e) => {
                    println!("error: {}", e);
                    continue;
                }
            };
            if choice <= 2 {
                println!();
                break choice;
            } else {
                println!("'{}' is not a valid choice", choice);
            }
        };

        stream.write_u8(choice)?;
        match choice {
            0 => {
                // User wants to exit
                break;
            }
            1 => {
                println!("Working with FheUint3");
                inner_loop_logic(&mut stream, &mut client_keys, fhe3_from_stin, (1 << 3))?;
            }
            2 => {
                println!("Working with FheUint16");
                inner_loop_logic::<FheUint16, u16, _>(
                    &mut stream,
                    &mut client_keys,
                    fhe16_from_stin,
                    (1 << 16),
                )?;
            }
            _ => panic!("Internal error, user was allowed to chose non supported type"),
        };
        println!("-----------------");
    }

    Ok(())
}
