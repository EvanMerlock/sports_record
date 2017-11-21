use std::net::{TcpListener, SocketAddr};
use std::thread;
use std::result::Result;
use std::path::Path;
use std::fs::File;
use std::sync::{Arc, Mutex};

use server::client_handling::*;
use server::web;
use server::{ ServerError, ServerErrorKind, sql };

use unsafe_code::init_av;

use rusqlite::Connection;
use iron::prelude::*;
use iron::Listening;
use router::Router;

pub struct RecordingServer {
    listener: Arc<TcpListener>,
    iron_server: Listening,
    client_handler: ClientHandler,
}

impl RecordingServer {

    pub fn new(ip_tuple: (SocketAddr, SocketAddr), db_loc: &Path) -> Result<RecordingServer, ServerError> {
        let tcp = try!(TcpListener::bind(ip_tuple.0));

        if !db_loc.exists() {
            let _ = try!(File::create(db_loc));
        }

        let database = sql::DatabaseRef::new(db_loc)?;

        let mut router = Router::new();
        router.get("/", web::web_handler::home_handler, "index");
        router.get("/videos/:query", web::web_handler::individual_video_handler, "query");

        let iron_serv_res = Iron::new(router).http(ip_tuple.1);

        init_av();
        let client_stream = try!(ClientStream::new(database));

        match iron_serv_res {
            Ok(item) => return Ok(RecordingServer { 
                listener: Arc::new(tcp), 
                iron_server: item, 
                client_handler: ClientHandler::new(Arc::new(Mutex::new(client_stream))),
            }),
            Err(_) => return Err(ServerError::new(ServerErrorKind::IronError)),
        }

    }

    pub fn get_client_handler(&self) -> &ClientHandler {
        &self.client_handler
    }

    pub fn start_handling_requests(&self) {
        let listener = self.listener.clone();
        let ip_sender = self.client_handler.get_internal();
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                            println!("Received Client: {:?}", stream.peer_addr());
                            let mut lock = ip_sender.lock().unwrap();
                            let _ = lock.add_client(stream);
                    }
                    Err(e) => println!("An error occurred: {}", e), 
                }
            }
        });
    }

}

pub struct ClientHandler(Arc<Mutex<ClientStream>>);

impl ClientHandler {
    pub fn new(internal: Arc<Mutex<ClientStream>>) -> ClientHandler {
        ClientHandler(internal)
    }

    pub fn start_recording(&self) {
        let lock = self.0.lock().unwrap();
        lock.start_recording();
    }

    pub fn stop_recording(&self) {
        let lock = self.0.lock().unwrap();
        lock.stop_recording();
    }

    pub fn clean_up(&self) {
        let mut lock = self.0.lock().unwrap();
        lock.clean_up();
    }

    pub fn remove_client(&self, info: SocketAddr) {
        let mut lock = self.0.lock().unwrap();
        lock.remove_client(info);
    }

    fn get_internal(&self) -> Arc<Mutex<ClientStream>> {
        self.0.clone()
    }
}