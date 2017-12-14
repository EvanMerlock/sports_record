use std::io::{ Write, Error, Read };
use std::fs::{ File };
use std::fs::read_dir;
use std::path::{ Path, PathBuf };
use std::net::SocketAddr;

use server::client_handling::*;

use serde_json;

use iron::response::*;
use iron::modifier;

#[derive(Debug)]
pub struct VideoPageHttpWriter {
    video_loc: PathBuf,
}

impl VideoPageHttpWriter {

    pub fn new(file_uuid: &str) -> VideoPageHttpWriter {
        VideoPageHttpWriter { video_loc: Path::new("./output/").join(file_uuid) }
    }

}

impl modifier::Modifier<Response> for VideoPageHttpWriter {

    fn modify(self, res: &mut Response) {
        res.body = Some(Box::new(self));
        ()
    }
    
}

impl WriteBody for VideoPageHttpWriter {

    fn write_body(&mut self, res: &mut Write) -> Result<(), Error> {
        for byte in try!(File::open(&self.video_loc)).bytes() {
            let _ = res.write(&[try!(byte)]);
        }
        Ok(())
    }

}

pub struct JsonOutputWriter {
    client_stream: WeakClientStream
}

impl JsonOutputWriter {
    pub fn new(client_stream: WeakClientStream) -> JsonOutputWriter {
        JsonOutputWriter {
            client_stream: client_stream
        }
    }
}

impl modifier::Modifier<Response> for JsonOutputWriter {
    fn modify(self, res: &mut Response) {
        res.body = Some(Box::new(self));
    }
}

impl WriteBody for JsonOutputWriter {
    fn write_body(&mut self, res: &mut Write) -> Result<(), Error> {

        let mut url_vec: Vec<(SocketAddr)> = Vec::new();
        let guard = self.client_stream.get_client_view().expect("pointer no longer exists. web server lived longer than client stream");
        let guard = guard.lock().expect("mutex is poisoned");

        for item in guard.iter() {
            // load WS url into struct
            url_vec.push((item.ws_url));
        }
        serde_json::to_writer(res, &url_vec)?;
        Ok(())
    }
}