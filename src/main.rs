use crate::Message::{ClientConnected, ClientDisconnected};
use std::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

struct Client {
    address: String,
}

impl Client {
    fn new(address: String) -> Self {
        Client { address }
    }
}

enum Message {
    ClientConnected { client: Client },
    ClientDisconnected,
}

fn server(msg_receiver: Receiver<Message>) {
    loop {
        let message = msg_receiver.recv().expect("Recevier channel hung up");

        match message {
            ClientConnected { client } => {
                println!("INFO: Client connected from address {}", client.address);
            }
            ClientDisconnected => {
                println!("INFO: Client disconnected");
            }
        }
    }
}

fn client(stream: TcpStream, msg_sender: Sender<Message>) {
    let client_address = match stream.peer_addr() {
        Ok(address) => address,
        Err(err) => {
            eprintln!("ERROR: Could not get the client address: {}", err);
            msg_sender
                .send(ClientDisconnected)
                .expect("Failed to send disconnect message");
            return;
        }
    };

    let client = Client::new(client_address.to_string());

    msg_sender
        .send(ClientConnected { client })
        .expect("Failed to send connection message");
}

fn main() {
    let listener =
        TcpListener::bind("127.0.0.1:5050").expect("Failed to create TCP listener on address");

    let (msg_sender, msg_receiver) = mpsc::channel::<Message>();

    thread::spawn(|| server(msg_receiver));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let msg_sender = msg_sender.clone();
                thread::spawn(move || client(stream, msg_sender));
            }
            Err(err) => eprintln!("ERROR: Failed to establish connection: {}", err),
        }
    }
}
