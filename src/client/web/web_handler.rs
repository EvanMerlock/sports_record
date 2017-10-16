use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};

use std::io;
use std::io::{Write, Read, BufRead};
use std::net::{TcpListener, TcpStream, SocketAddr, Shutdown};
use std::sync::{Arc, Mutex};
use std::thread;

use websocket::server::sync::{Server};
use websocket::client::sync::{Client};
use websocket::Message;


pub struct WebHandler {
    clients: Arc<Mutex<Vec<WebClient>>>,
    sender: Sender<Arc<Vec<u8>>>,
}

impl WebHandler {
    pub fn new(sock: SocketAddr) -> io::Result<WebHandler> {
        let server = Server::bind(sock)?;
        let (tx, rx) = channel::<Arc<Vec<u8>>>();
        let mut clients = Arc::new(Mutex::new(Vec::new()));
        let cloned_clients = Arc::clone(&clients);
        thread::spawn(move || {
            for upgrade_res in server {
                match upgrade_res {
                    Ok(upgrade) => {
                        if upgrade.protocols().contains(&"sports_record_jpeg_proto".to_owned()) {
                            let client = upgrade.use_protocol("sports_record_jpeg_proto").accept().expect("failed to open client");
                            cloned_clients.lock().expect("failed to lock mutex").push(WebClient::new(client));
                        } else {
                            upgrade.reject();
                        }
                    },
                    Err(e) => {},
                }
            };
        });
        let cloned_clients_two = clients.clone();
        thread::spawn(move || {
            for item in rx {
                for client in cloned_clients_two.lock().expect("failed to lock mutex").iter() {
                    client.send_frame(Arc::clone(&item));
                }
            }
        });
        Ok(WebHandler { clients: clients.clone(), sender: tx })
    }

    pub fn get_sender(&self) -> Sender<Arc<Vec<u8>>> {
        self.sender.clone()
    }
}

struct WebClient {
    stream: thread::JoinHandle<io::Result<Client<TcpStream>>>,
    data_sender: Sender<Arc<Vec<u8>>>,
}

impl WebClient {
    fn new(mut stream: Client<TcpStream>) -> WebClient {
        let (tx, rx) = channel::<Arc<Vec<u8>>>();
        let handle = thread::spawn(move || {
            let stream_connected = true;
            while stream_connected {
                match rx.try_recv() {
                    Ok(item) => { stream.send_message(&Message::binary(item.as_slice())); },
                    _ => {},
                }
            }
            stream.shutdown();
            Ok(stream)
        });
        WebClient {
            stream: handle,
            data_sender: tx
        }
    }

    fn send_frame(&self, item: Arc<Vec<u8>>) {
        self.data_sender.send(item);
    } 
}
