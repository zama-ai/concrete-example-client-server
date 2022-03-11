use concrete::prelude::*;
use concrete::{ClientKey, ConfigBuilder, FheUint16, FheUint3, FheUint3Parameters, ServerKey};
use std::error::Error;
use std::fs::File;
use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use std::path::Path;

const CLIENT_KEY_FILE_PATH: &'static str = "client_key.bin";
const SERVER_KEY_FILE_PATH: &'static str = "server_key.bin";

pub fn key_gen() -> Result<(ClientKey, ServerKey), Box<dyn Error>> {
    let client_key_path = Path::new(CLIENT_KEY_FILE_PATH);

    let client_keys: ClientKey = if client_key_path.exists() {
        println!("Reading client keys from {}", CLIENT_KEY_FILE_PATH);
        let mut file = BufReader::new(File::open(client_key_path)?);
        bincode::deserialize_from(file)?
    } else {
        println!(
            "No {} found, generating new keys and saving them",
            CLIENT_KEY_FILE_PATH
        );
        let config = ConfigBuilder::all_disabled()
            .enable_default_uint3()
            .enable_default_uint16()
            .build();
        let k = ClientKey::generate(config);
        let file = BufWriter::new(File::create(client_key_path)?);
        bincode::serialize_into(file, &k)?;

        k
    };

    let server_key_path = Path::new(SERVER_KEY_FILE_PATH);
    let server_keys: ServerKey = if server_key_path.exists() {
        println!("Reading server keys from {}", CLIENT_KEY_FILE_PATH);
        let mut file = BufReader::new(File::open(server_key_path)?);
        bincode::deserialize_from(file).unwrap()
    } else {
        println!(
            "No {} found, generating new keys and saving them",
            SERVER_KEY_FILE_PATH
        );
        let k = client_keys.generate_server_key();
        let file = BufWriter::new(File::create(server_key_path)?);
        bincode::serialize_into(file, &k).unwrap();

        k
    };

    Ok((client_keys, server_keys))
}

pub fn fhe3_from_stin(keys: &mut ClientKey) -> FheUint3 {
    loop {
        print!("Enter a number between [0; 7]: ");
        stdout().flush().unwrap();
        let mut buffer = String::new();
        let mut cin = stdin(); // We get `Stdin` here.
        cin.read_line(&mut buffer).unwrap();

        let value = match buffer.trim().parse::<u8>() {
            Ok(value) => value,
            Err(err) => {
                println!(
                    "'{}' is not a valid number: {}",
                    &buffer[..buffer.len() - 1],
                    err
                );
                continue;
            }
        };

        let fhe_value = match FheUint3::try_encrypt(value, keys) {
            Ok(v) => v,
            Err(_) => {
                println!("Value does not fit in a 3 bit integer");
                continue;
            }
        };

        return fhe_value;
    }
}

pub fn fhe16_from_stin(keys: &mut ClientKey) -> FheUint16 {
    loop {
        print!("Enter a number between [0; {}]: ", u16::MAX);
        stdout().flush().unwrap();

        let mut buffer = String::new();
        let mut cin = stdin(); // We get `Stdin` here.
        cin.read_line(&mut buffer).unwrap();

        let value = match buffer.trim().parse::<u16>() {
            Ok(value) => value,
            Err(err) => {
                println!(
                    "'{}' is not a valid number: {}",
                    &buffer[..buffer.len() - 1],
                    err
                );
                continue;
            }
        };

        return FheUint16::encrypt(value, keys);
    }
}

pub fn ask_for_exit() -> bool {
    println!("Exit ? [Y/n]");
    let mut cin = stdin(); // We get `Stdin` here.
    let mut buffer = String::with_capacity(5);
    loop {
        buffer.clear();
        cin.read_line(&mut buffer).unwrap();

        let trimmed = buffer.trim_end();
        if trimmed.is_empty() || trimmed == "Y" || trimmed == "y" {
            return true;
        } else if trimmed == "n" {
            return false;
        }
    }
}
