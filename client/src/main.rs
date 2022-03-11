#![allow(warnings)]
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;

use bincode;
use byteorder::WriteBytesExt;
use std::io::prelude::*;
use std::net::TcpStream;

use crate::details::{ask_for_exit, fhe16_from_stin, fhe3_from_stin, key_gen};
use concrete::prelude::*;
use concrete::{FheUint16, FheUint3};

mod details;

fn main() -> Result<(), Box<dyn Error>> {
    let (mut client_keys, mut server_keys) = key_gen()?;

    println!("[Client] ----> [Server]: Connecting to server");
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;

    {
        println!("[Client] ----> [Server]: Sending Bootstrap Keys to server");
        bincode::serialize_into(&mut stream, &server_keys)?;
        println!("[Client] ----> [Server]: transaction successful");
    }

    loop {
        stream.write_u8(1)?;

        {
            let a = fhe3_from_stin(&mut client_keys);
            let b = fhe3_from_stin(&mut client_keys);
            let c = fhe3_from_stin(&mut client_keys);

            println!("[Client] ----> [Server]: Sending a, b, c");
            bincode::serialize_into(&mut stream, &a)?;
            bincode::serialize_into(&mut stream, &b)?;
            bincode::serialize_into(&mut stream, &c)?;

            println!("[Client] <---- [Server]: Receiving result");
            let result: FheUint3 = bincode::deserialize_from(&mut stream).unwrap();

            let clear_result = result.decrypt(&mut client_keys);
            println!("The result is: {}", clear_result);
        }

        {
            let a = fhe16_from_stin(&mut client_keys);
            let b = fhe16_from_stin(&mut client_keys);
            let c = fhe16_from_stin(&mut client_keys);

            println!("[Client] ----> [Server]: Sending a, b, c");
            bincode::serialize_into(&mut stream, &a)?;
            bincode::serialize_into(&mut stream, &b)?;
            bincode::serialize_into(&mut stream, &c)?;

            println!("[Client] <---- [Server]: Receiving result");
            let result: FheUint16 = bincode::deserialize_from(&mut stream).unwrap();
            let clear_result: u16 = result.decrypt(&mut client_keys);

            println!("The result is: {}", clear_result);
        }

        let should_exit = ask_for_exit();
        if should_exit {
            stream.write_u8(0)?;
            break;
        }
    }

    Ok(())
}
