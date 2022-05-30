use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EventType {
    None = 0,
    Progress = 1,
    ServerWelcome = 2,
    FileMetadata = 3,
    ConnectedToRelay = 4,
    Code = 5,
}

impl Default for EventType {
    fn default() -> Self {
        EventType::None
    }
}


#[derive(Default, serde::Serialize, serde::Deserialize)]
#[wasm_bindgen]
pub struct Event {
    event_type: EventType,
    server_welcome_message: String,
    file_name: String,
    file_size: u64,
    progress_current: u64,
    progress_total: u64,
    code: String,
}

// helper methods

pub fn server_welcome(server_welcome_message: String) -> Event {
    Event {
        event_type: EventType::ServerWelcome,
        server_welcome_message,
        ..Event::default()
    }
}

pub fn file_metadata(file_name: String, file_size: u64) -> Event {
    Event {
        event_type: EventType::FileMetadata,
        file_name,
        file_size,
        ..Event::default()
    }
}

pub fn connected() -> Event {
    Event {
        event_type: EventType::ConnectedToRelay,
        ..Event::default()
    }
}

pub fn progress(progress_current: u64, progress_total: u64) -> Event {
    Event {
        event_type: EventType::Progress,
        progress_current,
        progress_total,
        ..Event::default()
    }
}

pub fn code(code: String) -> Event {
    Event {
        event_type: EventType::Code,
        code,
        ..Event::default()
    }
}