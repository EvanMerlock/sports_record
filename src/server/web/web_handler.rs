
use server::web::body_writer;

use iron::prelude::*;
use iron::headers::ContentType;
use iron::status;
use router::Router;
use iron::mime::{Mime, TopLevel, SubLevel};

pub fn home_handler(_: &mut Request) -> IronResult<Response> {
    let mut res: Response = Response::with((status::Ok, body_writer::VideoHttpWriter::new()));
    res.headers.set(ContentType::html());
    Ok(res)
}

pub fn individual_video_handler(req: &mut Request) -> IronResult<Response> {
    let ref query = req.extensions.get::<Router>();
    match *query {
        Some(q) => {
            let uuid: &str = q.find("query").unwrap_or("bad_file");
            let mut res: Response = Response::with((status::Ok, body_writer::VideoPageHttpWriter::new(uuid)));
            res.headers.set(ContentType("video/mp4".parse().unwrap()));
            Ok(res)
        }
        None => {
            let res: Response = Response::with((status::NotFound, "Not Found!"));
            Ok(res)
        }
    }
}