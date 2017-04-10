pub struct Messenger {
    pub current_status: CurrentServerStatus,
}

impl Messenger {
    pub fn new() -> Messenger {
        Messenger { current_status: CurrentServerStatus::NotRecording }
    }

    pub fn set_status(&mut self, new_status: CurrentServerStatus) {
        self.current_status = new_status;
    }
}

#[derive(Debug)]
pub enum CurrentServerStatus {
    StartingRecording,
    Recording,
    EndingRecording,
    NotRecording,
}