use byteorder::ReadBytesExt;
use concrete::prelude::*;
use concrete::{set_server_key, FheUint16, FheUint3, ServerKey};
use std::net::{TcpListener, TcpStream};
use std::ops::{Add, Mul};

fn fhe_computation<'a, T>(a: &'a T, b: &'a T, c: &'a T) -> T
where
    &'a T: Add<&'a T, Output = T>,
    T: Mul<&'a T, Output = T>,
{
    (a + b) * c
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    println!("[Server] <---- [Client]: Receiving server keys from client");
    {
        let server_keys: ServerKey = bincode::deserialize_from(&mut stream).unwrap();
        set_server_key(server_keys);
    }

    loop {
        let choice = stream.read_u8()?;
        if choice == 0 {
            println!("[Server] <---- [Client]: User said good bye");
            break;
        }

        {
            println!("[Server] <---- [Client]: Receiving a, b, c");
            let a: FheUint3 = bincode::deserialize_from(&mut stream).unwrap();
            let b: FheUint3 = bincode::deserialize_from(&mut stream).unwrap();
            let c: FheUint3 = bincode::deserialize_from(&mut stream).unwrap();

            print!("Computing...");
            let result = fhe_computation(&a, &b, &c);
            println!("done.");
            println!("[Server] ----> [Client]: Sending Result");
            bincode::serialize_into(&mut stream, &result).unwrap();
        }

        {
            println!("[Server] <---- [Client]: Receiving a, b, c");
            let a: FheUint16 = bincode::deserialize_from(&mut stream).unwrap();
            let b: FheUint16 = bincode::deserialize_from(&mut stream).unwrap();
            let c: FheUint16 = bincode::deserialize_from(&mut stream).unwrap();

            print!("Computing...");
            let result = fhe_computation(&a, &b, &c);
            println!("done.");
            println!("[Server] ----> [Client]: Sending Result");
            bincode::serialize_into(&mut stream, &result).unwrap();
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
