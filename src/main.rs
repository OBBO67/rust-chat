use crate::Message::{ClientConnected, ClientDisconnected, ClientMessage};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    ops::Deref,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
};

struct Client {
    address: String,
    stream: Arc<TcpStream>,
}

impl Client {
    fn new(address: String, stream: Arc<TcpStream>) -> Self {
        Client { address, stream }
    }
}

enum Message {
    ClientConnected { client: Client },
    ClientDisconnected,
    ClientMessage { msg: String },
}

fn server(msg_receiver: Receiver<Message>) {
    loop {
        let message = msg_receiver.recv().expect("Recevier channel hung up");

        match message {
            ClientConnected { client } => {
                println!("INFO: Client connected from address {}", client.address);
                let response = "Thanks for connecting\n";
                client
                    .stream
                    .deref()
                    .write_all(response.as_bytes())
                    .expect("Failed to respond to client");
            }
            ClientDisconnected => {
                println!("INFO: Client disconnected");
            }
            ClientMessage { msg } => {
                println!("{}", msg);
            }
        }
    }
}

fn client(stream: Arc<TcpStream>, msg_sender: Sender<Message>) {
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

    let client = Client::new(client_address.to_string(), stream.clone());

    msg_sender
        .send(ClientConnected { client })
        .expect("Failed to send connection message");

    let mut buffer = Vec::new();

    // buffer needs to a certain length before reading from the TCP stream
    buffer.resize(64, 0);

    loop {
        let bytes_read = stream
            .as_ref()
            .read(&mut buffer)
            .map_err(|err| {
                eprintln!("ERROR: error reading the message from the client {}", err);
                msg_sender
                    .send(Message::ClientDisconnected)
                    .expect("Failed to send message disconnected message to server");
            })
            .expect("Unable to get the Arc reference ");

        if bytes_read > 0 {
            println!("Bytes read {}", bytes_read);
            msg_sender
                .send(Message::ClientMessage {
                    msg: String::from_utf8(buffer.clone()).unwrap(),
                })
                .expect("Failed to send message to server");
        }
    }
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
                let stream = Arc::new(stream);
                thread::spawn(move || client(stream, msg_sender));
            }
            Err(err) => eprintln!("ERROR: Failed to establish connection: {}", err),
        }
    }
}
