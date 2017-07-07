use std::marker::{Send, Sync};

#[derive(Debug, PartialEq)]
pub enum ClientStatusFlag {
    StartRecording,
    StopRecording,
}

unsafe impl Send for ClientStatusFlag {}
unsafe impl Sync for ClientStatusFlag {}