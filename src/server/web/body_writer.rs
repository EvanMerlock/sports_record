use std::io::{ Write, Error, Read };
use std::fs::{ File };
use std::fs::read_dir;
use std::path::{ Path, PathBuf };

use iron::response::*;
use iron::modifier;

#[derive(Debug)]
pub struct VideoHttpWriter {

    output_folder: &'static Path,

}

impl VideoHttpWriter {

    pub fn new() -> VideoHttpWriter {
        VideoHttpWriter { output_folder: Path::new("./output/") }
    }

}

impl WriteBody for VideoHttpWriter {
     fn write_body(&mut self, res: &mut Write) -> Result<(), Error> {
         for file in try!(read_dir(self.output_folder)) {
             let file = try!(file);
             let path_ref = file.path();

             match path_ref.extension() {
                 Some(ext) => {
                     if ext == "mp4" {
                        let path = path_ref.to_str();
                        match path {
                            Some(string) => {
                                let _ = res.write(b"<i>");
                                let _ = res.write(string.as_bytes());
                                let _ = res.write(b"</i><br>");
                                ()
                            }
                            None => { 
                                let _ = res.write(b"<i>Error</i>");
                                ()
                            }
                        }
                     }
                 }
                 None => {
                    let _ = res.write(b"<i>Error</i>");
                    ()
                 }
             }
         }

         Ok(())
     }

}

impl modifier::Modifier<Response> for VideoHttpWriter {

    fn modify(self, res: &mut Response) {
        res.body = Some(Box::new(self));
        ()
    }
    
}



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