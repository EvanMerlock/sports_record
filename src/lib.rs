#[macro_use]
extern crate serde_derive;

extern crate iron;
extern crate router;
extern crate rusqlite;
extern crate ffmpeg_sys;
extern crate uuid;
extern crate magick_rust;
extern crate serde;
extern crate serde_json;
extern crate messenger_plus;
extern crate time;
extern crate libc;
extern crate rmp_serde;

pub mod server;
pub mod client;
pub mod unsafe_code;
pub mod config;
pub mod networking;