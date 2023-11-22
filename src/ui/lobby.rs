use askama::Template;
use axum::{
    extract::{Form, State},
    response::IntoResponse,
};

use super::utils::{render_or_else, response_or};
use crate::{game::settings::Settings, server::MutState};

#[derive(Template)]
#[template(path = "lobby/rooms.html")]
struct Rooms<'a> {
    rooms: Vec<Room<'a>>,
}
pub async fn get_rooms(State(state): State<MutState>) -> String {
    render_or_else(
        &Rooms {
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
        "could not render room list",
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
    pub player_count: usize,
}
pub async fn post_rooms(
    State(state): State<MutState>,
    Form(new_room): Form<NewRoom>,
) -> impl IntoResponse {
    let rs = &mut state.lock().await;

    let settings = Settings::new(new_room.player_count);
    let room_id = rs.add_room(settings, &new_room.name);

    response_or(
        || async {
            let room = rs.get_room(&room_id)?;
            Ok(Room {
                name: &room.name,
                id: &room_id,
            }
            .render()?)
        },
        "could not render room",
    )
    .await
}
