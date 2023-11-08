use askama::Template;
use axum::{
    extract::{Form, Path, State},
    response::Html,
};

use crate::types::MutState;

use super::{page, render_or_else};

pub async fn room(Path(room_id): Path<usize>, State(state): State<MutState>) -> Html<String> {
    match &mut state.lock().await.rooms.get_mut(&room_id) {
        Some(room) => page("RDND - room", {
            if let Err(e) = room.game.next().await {
                println!("{e}");
            }

            &format!(
                "Room: {}<br>{}",
                room.name,
                room.game.messages().join("<br>")
            )
        }),
        None => page("RDND - room", "Room not found"),
    }
}

#[derive(Template)]
#[template(path = "../templates/rooms/room.html")]
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

#[derive(Template)]
#[template(path = "../templates/rooms/rooms.html")]
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
