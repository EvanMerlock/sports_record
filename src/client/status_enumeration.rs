use std::marker::{Send, Sync};

#[derive(Debug, PartialEq)]
pub enum ClientStatusFlag {
    StartRecording,
    StopRecording,
    ServerQuit,
}

unsafe impl Send for ClientStatusFlag {}
unsafe impl Sync for ClientStatusFlag {}