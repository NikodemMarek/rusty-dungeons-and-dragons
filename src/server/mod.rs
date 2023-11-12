use std::{collections::HashMap, sync::Arc};

pub mod room;

use crate::server::room::Room;

#[derive(Debug)]
pub struct AppState {
    pub rooms: HashMap<usize, Arc<Room>>,
    id: usize,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            id: 0,
        }
    }

    pub fn add_room(&mut self, name: &str) -> usize {
        self.rooms.insert(self.id, Arc::new(Room::new(name)));
        self.id += 1;
        self.id - 1
    }
}

pub type MutState = std::sync::Arc<tokio::sync::Mutex<AppState>>;
