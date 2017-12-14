use std::net::{TcpListener, TcpStream, SocketAddr};
use std::thread;
use std::result::Result;
use std::path::Path;
use std::fs::File;
use std::sync::{Arc, Mutex};

use server::client_handling::*;
use server::web;
use server::{ ServerError, ServerErrorKind, sql };

use ffmpeg_common::unsafe_code::init_av;

use rusqlite::Connection;
use iron::prelude::*;
use iron::Listening;
use router::Router;
use rand;
use rand::Rng;

use server::ServerConfiguration;

pub struct RecordingServer {
    listener: Arc<TcpListener>,
    iron_server: Listening,
    client_handler: ClientStream,
    control_panel_key: String,
}

impl RecordingServer {

    pub fn new(server_conf: ServerConfiguration) -> Result<RecordingServer, ServerError> {
        let tcp = TcpListener::bind(server_conf.get_clip_server_port())?;
        let db_loc = Path::new(server_conf.get_output_directory());
        let db_loc = db_loc.join(server_conf.get_database_name());

        if !db_loc.exists() {
            let _ = File::create(&db_loc)?;
        }

        let database = sql::DatabaseRef::new(&db_loc)?;

        init_av();
        let client_stream = try!(ClientStream::new(database, server_conf.get_output_directory().to_str().unwrap().to_owned()));


        let mut router = Router::new();
        router.get("/", web::web_handler::control_panel_handler, "index");

        let mut asset_chain = Chain::new(web::web_handler::asset_handler);
        let weak_client = client_stream.get_weak();
        asset_chain.link_before(move |req: &mut Request| { req.extensions.insert::<WeakClientStream>(weak_client.clone()); Ok(()) } );
        router.get("/dist/:query", asset_chain, "asset_handling");
        router.get("/videos/:query", web::web_handler::individual_video_handler, "query");

        let iron_serv_res = Iron::new(router).http(server_conf.get_web_server_port());

        let mut rng = rand::thread_rng();
        let ascii_chars: String = rng.gen_ascii_chars().take(20).fold(String::from(""), |mut init: String, item: char| { init.push(item); init });

        match iron_serv_res {
            Ok(item) => return Ok(RecordingServer { 
                listener: Arc::new(tcp), 
                iron_server: item, 
                client_handler: client_stream,
                control_panel_key: ascii_chars,
            }),
            Err(_) => return Err(ServerError::new(ServerErrorKind::IronError)),
        }

    }

    pub fn get_client_handler(&self) -> ClientStream {
        self.client_handler.clone()
    }

    pub fn start_handling_requests(&self) {
        let listener = self.listener.clone();
        let mut ip_sender = self.client_handler.clone();
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                            println!("Received Client: {:?}", stream.peer_addr());
                            let _ = ip_sender.add_client(stream);
                    }
                    Err(e) => println!("An error occurred: {}", e), 
                }
            }
        });
    }

}