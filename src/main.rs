use std::{
    fs::File,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};



fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let tunnel_client = stream.unwrap();
        thread::spawn(move || {
            match TcpStream::connect(("127.0.0.1", 8080)) {
                Ok(target_server) => {
                    let tunnel_client = Arc::new(Mutex::new(tunnel_client));
                    let target_server = Arc::new(Mutex::new(target_server));

                    let tunnel_client_clone = tunnel_client.clone();
                    let target_server_clone = target_server.clone();
                    let handle = thread::spawn(move || {
                        // Forward data from the source to the target.

                        let mut source = tunnel_client_clone.lock().unwrap();
                        let mut target = target_server_clone.lock().unwrap();

                        if let Err(e) = forward(&mut source, &mut target, true) {
                            eprintln!("error: {}", e);
                        }

                    });
                    let tunnel_client_clone2 = tunnel_client.clone();
                    let target_server_clone2 = target_server.clone();
                    // Create a separate thread to handle forwarding from target to source.
                    let handle2 = thread::spawn(move || {
                        // Forward data from the target to the source.
                        let mut source = target_server_clone2.lock().unwrap();
                        let mut target = tunnel_client_clone2.lock().unwrap();
                        if let Err(e) = forward(&mut source, &mut target, false) {
                            eprintln!("Error forwarding data: {}", e);
                        }
                    });

                    handle.join().expect("Thread 1 panicked");
                    handle2.join().expect("Thread 2 panicked");

                    fn forward(
                        source: &mut TcpStream,
                        target: &mut TcpStream,
                        tunnel_is_source: bool,
                    ) -> io::Result<()> {
                        let mut buffer = [0; 1024];
                        loop {
                            match source.read(&mut buffer) {
                                Ok(0) => {
                                    return Ok(());
                                }
                                Ok(bytes_read) => {
                                    target.write(&buffer[..bytes_read]).unwrap();
                                    if tunnel_is_source
                                        && buffer[bytes_read - 2] == 0x0d
                                        && buffer[bytes_read - 1] == 0x0a
                                    {
                                        return Ok(());
                                    }
                                }
                                Err(e) => {
                                    panic!("READ error: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        });
    }
}

fn handle_request(_tunnel_client: &mut TcpStream) {}
