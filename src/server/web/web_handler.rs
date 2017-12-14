
use server::web::body_writer;
use server::client_handling::WeakClientStream;

use iron::prelude::*;
use iron::headers::ContentType;
use iron::status;
use router::Router;

const index_bytes: &'static [u8] = include_bytes!("../../../html/server/index.html");
const javascript_package: &'static [u8] = include_bytes!("../../../html/server/dist/build.js");

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

pub fn control_panel_handler(req: &mut Request) -> IronResult<Response> {
    let ref query = req.extensions.get::<Router>();
    match *query {
        Some(q) => {
            let mut res = Response::with((status::NotFound, index_bytes));
            res.headers.set(ContentType("text/html".parse().unwrap()));
            Ok(res)
        },
        None => {
            let res: Response = Response::with((status::NotFound, "Not Found!"));
            Ok(res)
        },
    }
}

pub fn asset_handler(req: &mut Request) -> IronResult<Response> {
    let ref query = req.extensions.get::<Router>();
    match *query {
        Some(q) => {
            let location: &str = q.find("query").unwrap_or("");
            match location {
                "" => Ok(Response::with((status::NotFound, "Under Construction, Blank Asset"))),
                "build.js" => { 
                    let mut res = Response::with((status::Ok, javascript_package));
                    res.headers.set(ContentType("text/javascript".parse().unwrap()));
                    Ok(res)
                },
                "connected_servers.json" => {
                    let mut res = Response::with((status::Ok, body_writer::JsonOutputWriter::new(req.extensions.get::<WeakClientStream>().expect("failed to get client stream").clone())));
                    res.headers.set(ContentType("application/json".parse().unwrap()));
                    Ok(res)
                },
                _ => Ok(Response::with((status::NotFound, "Under Construction, Unknown Asset"))),
            }
        },
        None => {
            let res: Response = Response::with((status::NotFound, "Not Found!"));
            Ok(res)
        },
    }
}