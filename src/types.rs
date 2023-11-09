use axum::extract::ws::WebSocket;
use std::collections::HashMap;

use crate::game::Game;

#[derive(Debug)]
pub struct Client {
    pub name: String,
    socket: WebSocket,
}

#[derive(Debug)]
pub struct Room {
    pub name: String,
    pub clients: Vec<Client>,
    pub game: Game,
}
impl Room {
    pub fn new(name: String) -> Self {
        Self {
            name,
            clients: Vec::new(),
            game: Game::new(),
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
        self.rooms.insert(self.id, Room::new(name));
        self.id += 1;
        self.id
    }
}

pub type MutState = std::sync::Arc<tokio::sync::Mutex<AppState>>;
