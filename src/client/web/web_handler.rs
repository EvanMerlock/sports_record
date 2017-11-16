use std::sync::mpsc::{channel, Sender};

use std::io;
use std::net::{TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;

use websocket::server::sync::{Server};
use websocket::client::sync::{Client};
use websocket::Message;

use client::web::body_writer;

use base64;
use iron::prelude::*;
use iron::Listening;
use router::Router;


pub struct WebHandler {
    clients: Arc<Mutex<Vec<WebClient>>>,
    sender: Sender<Arc<Vec<u8>>>,
    web_server: Listening,
}

impl WebHandler {
    pub fn new(sock: (SocketAddr, SocketAddr)) -> io::Result<WebHandler> {
        let server = Server::bind(sock.0)?;
        
        let mut router = Router::new();
        router.get("/", body_writer::stream_handler, "index");

        let iron_server_res = Iron::new(router).http(sock.1).expect("failed to start stream server");

        let (tx, rx) = channel::<Arc<Vec<u8>>>();
        let clients = Arc::new(Mutex::new(Vec::new()));
        let cloned_clients = Arc::clone(&clients);
        thread::spawn(move || {
            for upgrade_res in server {
                match upgrade_res {
                    Ok(upgrade) => {
                        if upgrade.protocols().contains(&"sports_record_jpeg_proto".to_owned()) {
                            let client = upgrade.use_protocol("sports_record_jpeg_proto").accept().expect("failed to open client");
                            cloned_clients.lock().expect("failed to lock mutex").push(WebClient::new(client));
                        } else {
                            let _ = upgrade.reject();
                        }
                    },
                    Err(_) => {},
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
        Ok(WebHandler { clients: clients.clone(), sender: tx, web_server: iron_server_res })
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
                    Ok(item) => { let _ = stream.send_message(&Message::text(format!("data:image/png;base64,{}", base64::encode(item.as_slice())))); },
                    _ => {},
                }
            }
            let _ = stream.shutdown();
            Ok(stream)
        });
        WebClient {
            stream: handle,
            data_sender: tx
        }
    }

    fn send_frame(&self, item: Arc<Vec<u8>>) {
        let _ = self.data_sender.send(item);
    } 
}
