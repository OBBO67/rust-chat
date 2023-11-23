use crate::Message::{ClientConnected, ClientDisconnected, ClientMessage};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    ops::Deref,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
};

#[derive(Clone)]
struct Client {
    address: SocketAddr,
    stream: Arc<TcpStream>,
}

impl Client {
    fn new(address: SocketAddr, stream: Arc<TcpStream>) -> Self {
        Client { address, stream }
    }
}

enum Message {
    ClientConnected {
        client: Client,
    },
    ClientDisconnected {
        client_address: SocketAddr,
    },
    ClientMessage {
        client_address: SocketAddr,
        msg: String,
    },
}

fn server(msg_receiver: Receiver<Message>) {
    let mut connected_clients = HashMap::new();

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
                connected_clients.insert(client.address, client.clone());
            }
            ClientDisconnected { client_address } => {
                println!("INFO: Client {} disconnected", client_address);
                connected_clients.remove(&client_address);
            }
            ClientMessage {
                client_address: sender_address,
                msg,
            } => {
                println!("INFO: Message received from client {}", sender_address);
                connected_clients
                    .iter()
                    .filter(|(&key, &_)| key != sender_address)
                    .for_each(|(&_, &ref client)| {
                        client
                            .stream
                            .deref()
                            .write_all(msg.as_bytes())
                            .expect("Unable to send message to other clients");
                    });
            }
        }
    }
}

fn client(stream: Arc<TcpStream>, msg_sender: Sender<Message>) {
    let client_address = match stream.peer_addr() {
        Ok(address) => address,
        Err(err) => {
            eprintln!("ERROR: Could not get the client address: {}", err);
            return;
        }
    };

    let client = Client::new(client_address, stream.clone());

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
                    .send(Message::ClientDisconnected { client_address })
                    .expect("Failed to send message disconnected message to server");
            })
            .unwrap();

        println!("INFO: number of bytes received: {}", bytes_read);

        if bytes_read > 0 {
            println!("Bytes read {}", bytes_read);
            let message_bytes: Vec<_> = buffer.clone().into_iter().filter(|b| *b >= 32).collect();
            msg_sender
                .send(Message::ClientMessage {
                    client_address,
                    msg: String::from_utf8(message_bytes).unwrap(),
                })
                .expect("Failed to send message to server");
        } else {
            msg_sender
                .send(Message::ClientDisconnected { client_address })
                .expect("Unable to send client disconnet message to server");
            break;
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
