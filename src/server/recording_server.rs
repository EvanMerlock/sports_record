use iron::prelude::*;
use iron::Listening;
use router::Router;

use std::net::{TcpListener, SocketAddr};
use std::thread;
use std::result::Result;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::sync::{Arc};
use std::sync::mpsc::{Sender};

use server::io_handler;

use server::client_handling::*;
use server::web;
use server::{ ServerError, ServerErrorKind };

use rusqlite::Connection;

pub struct RecordingServer {
    listener: Arc<TcpListener>,
    iron_server: Listening,
    sql_db_location: PathBuf,
    ip_sender: Sender<ClientIPInformation>,
    instruction_sender: Sender<RecordingInstructions>,
    client_stream_handler: ClientStream,
}

impl RecordingServer {

    pub fn new(ip_tuple: (SocketAddr, SocketAddr), db_loc: &Path) -> Result<RecordingServer, ServerError> {
        let tcp = try!(TcpListener::bind(ip_tuple.0));

        if !db_loc.exists() {
            let _ = try!(File::create(db_loc));
        }

        let connection = try!(Connection::open(db_loc.to_owned()));
        try!(connection.execute("CREATE TABLE IF NOT EXISTS videos (id INTEGER)", &[]));

        let mut router = Router::new();
        router.get("/", web::web_handler::home_handler, "index");
        router.get("/:query", web::web_handler::individual_video_handler, "query");

        let iron_serv_res = Iron::new(router).http(ip_tuple.1);

        let client_stream = try!(ClientStream::new());

        match iron_serv_res {
            Ok(item) => return Ok(RecordingServer { 
                listener: Arc::new(tcp), 
                iron_server: item, 
                sql_db_location: db_loc.to_owned(),
                ip_sender: client_stream.get_ip_information_sender(),
                instruction_sender: client_stream.get_instruction_sender(),
                client_stream_handler: client_stream,
            }),
            Err(_) => return Err(ServerError::new(ServerErrorKind::IronError)),
        }

    }

    pub fn get_instruction_sender(&self) -> Sender<RecordingInstructions> {
        self.instruction_sender.clone()
    }

    pub fn get_ip_sender(&self) -> Sender<ClientIPInformation> {
        self.ip_sender.clone()
    }

    pub fn start_handling_requests(&self) {
        let listener = self.listener.clone();
        let ip_sender = self.ip_sender.clone();
        thread::spawn(move || {
            for stream in listener.incoming() {
                let ip_sender = ip_sender.clone();
                match stream {
                    Ok(stream) => {
                        thread::spawn(move || {
                            let _ = io_handler::stream_handler(stream, ip_sender);
                        });
                    }
                    Err(e) => { println!("Err occured: {}", e) }
                }
            }
        });
    }

}