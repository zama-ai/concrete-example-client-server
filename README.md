# Concrete Client/Server example

This concrete example demonstrates the use of:

* Building Client Server architecture
* Communicating via TcpStream
* Using serialization to exchange encrypted data
* Using serialization to save a client key locally and save processing time
* Creating a generic function for use with different Fhe types
* Simple multithreading on the server side to handle multiple clients at the same time

for more information go to Concrete [documentation](https://docs.zama.ai/concrete/how-to/client_server)


# Running

You will need to start the server before the client,
and be sure to run in release mode to get good performances.

On the very first run the client will generate keys and save them to a `client_key.bin`
and `server_key.bin` files.

```console
# In a terminal
cargo run --release -p server

# In another terminal
cargo run --release -p client
```