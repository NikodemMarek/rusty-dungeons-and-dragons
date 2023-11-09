use askama::Template;
use axum::{
    extract::{Path, State},
    response::Html,
};

use crate::types::MutState;
use crate::ui::page;

use super::render_or_else;

#[derive(Template)]
#[template(path = "room/page.html")]
struct Page<'a> {
    name: &'a str,
    messages: &'a [Message<'a>],
}
pub async fn room(Path(room_id): Path<usize>, State(state): State<MutState>) -> Html<String> {
    match &mut state.lock().await.rooms.get_mut(&room_id) {
        Some(room) => page("RDND - room", {
            // if let Err(e) = room.game.next().await {
            //     println!("{e}");
            // }

            &render_or_else(
                Page {
                    name: &room.name,
                    messages: &room
                        .game
                        .messages()
                        .iter()
                        .map(Into::into)
                        .collect::<Vec<Message>>(),
                },
                "Couldn't render",
            )
        }),
        None => page("RDND - room", "Room not found"),
    }
}

#[derive(Template)]
#[template(path = "room/message.html")]
struct Message<'a> {
    content: &'a str,
}
impl<'a> From<&'a crate::game::Message> for Message<'a> {
    fn from(msg: &'a crate::game::Message) -> Self {
        match msg {
            crate::game::Message::Master(m) => Self {
                content: &m.content,
            },
            crate::game::Message::Player(m) => Self {
                content: &m.content,
            },
        }
    }
}
