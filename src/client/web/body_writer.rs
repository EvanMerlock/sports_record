use std::io::{ Write, Error };

use iron::prelude::*;
use iron::headers::ContentType;
use iron::status;
use iron::modifier;
use iron::response::WriteBody;

use liquid;
use liquid::{Context, Value, Renderable};

const STREAM_TEMPLATE_PAGE: &'static str = include_str!("../../../html/video_stream.html");

pub fn stream_handler(_: &mut Request) -> IronResult<Response> {
    let mut res: Response = Response::with((status::Ok, StreamRenderer));
    res.headers.set(ContentType::html());
    Ok(res)
}

pub struct StreamRenderer;

impl WriteBody for StreamRenderer {
    fn write_body(&mut self, res: &mut Write) -> Result<(), Error> {
        let template = liquid::parse(STREAM_TEMPLATE_PAGE, Default::default()).expect("failed to parse template");
        let mut context = Context::new();
        context.set_val("pagetitle", Value::str("Camera Stream"));
        context.set_val("socketproto", Value::str("sports_record_jpeg_proto"));
        context.set_val("socketaddress", Value::str("ws://127.0.0.1:4000"));
        let output = template.render(&mut context).expect("failed to render").expect("failed to render");
        res.write_all(output.as_ref())?;
        Ok(())
    }
}

impl modifier::Modifier<Response> for StreamRenderer {
    fn modify(self, res: &mut Response) {
        res.body = Some(Box::new(self));
        ()
    }
}