use askama::Template;
use axum::{
    extract::{Form, Path, State},
    response::Html,
};

use crate::types::MutState;

use super::render_or_else;

#[derive(Template)]
#[template(path = "lobby/rooms.html")]
struct Rooms<'a> {
    rooms: Vec<Room<'a>>,
}
pub async fn get_rooms(State(state): State<MutState>) -> String {
    render_or_else(
        Rooms {
            rooms: state
                .lock()
                .await
                .rooms
                .iter()
                .map(|r| Room {
                    name: &r.1.name,
                    id: r.0,
                })
                .collect::<Vec<Room>>(),
        },
        "Couldn't render room list",
    )
}

#[derive(Template)]
#[template(path = "lobby/room.html")]
struct Room<'a> {
    name: &'a str,
    id: &'a usize,
}
#[derive(Debug, serde::Deserialize)]
pub struct NewRoom {
    pub name: String,
}
pub async fn post_rooms(State(state): State<MutState>, Form(new_room): Form<NewRoom>) -> String {
    let rs = &mut state.lock().await;

    let room_id = rs.add_room(new_room.name);
    match rs.rooms.get(&room_id) {
        Some(room) => render_or_else(
            Room {
                name: &room.name,
                id: &room_id,
            },
            "Couldn't render room",
        ),
        _ => String::from("Couldn't add a room"),
    }
}
