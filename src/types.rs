use axum::extract::ws::WebSocket;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Room {
    pub name: String,
    pub clients: Vec<WebSocket>,
}
impl Room {
    pub fn new(name: String) -> Self {
        Self {
            name,
            clients: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct AppState {
    pub rooms: HashMap<usize, Room>,
    id: usize,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            id: 0,
        }
    }

    pub fn add_room(&mut self, name: String) -> usize {
        self.id += 1;
        self.rooms.insert(self.id, Room::new(name));
        self.id
    }
}

pub type MutState = std::sync::Arc<std::sync::Mutex<AppState>>;
