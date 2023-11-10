use std::net::TcpListener;

fn main() {
    let listener =
        TcpListener::bind("127.0.0.1:5050").expect("Failed to create TCP listener on address");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer_address = stream
                    .peer_addr()
                    .expect("Could not determine the peer address");
                println!(
                    "INFO: Connection successfully established with {}",
                    peer_address
                )
            }
            Err(err) => eprintln!("ERROR: Failed to establish connection: {}", err),
        }
    }
}
