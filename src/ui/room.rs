use askama::Template;

use axum::{
    extract::{connect_info::ConnectInfo, ws, Path, State},
    response::{Html, IntoResponse},
    TypedHeader,
};
use std::net::SocketAddr;

use crate::types::MutState;
use crate::ui::page;

use super::render_or_else;

#[derive(Template)]
#[template(path = "room/page.html")]
struct Page<'a> {
    id: usize,
    name: &'a str,
    messages: &'a [Message<'a>],
}
pub async fn room(Path(room_id): Path<usize>, State(state): State<MutState>) -> Html<String> {
    match state.lock().await.rooms.get_mut(&room_id) {
        Some(room) => page("RDND - room", {
            &render_or_else(
                Page {
                    id: room_id,
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

pub async fn join_room(
    Path(room_id): Path<usize>,
    State(state): State<MutState>,

    ws: ws::WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        if let Some(room) = state.lock().await.rooms.get_mut(&room_id) {
            // dbg!(&ws, &user_agent, addr);
            let client = if let Some(TypedHeader(user_agent)) = user_agent {
                user_agent.to_string()
            } else {
                String::from("unknown browser")
            };

            println!("`{client}` connected");
            room.add_client(crate::types::Client::new(socket, client))
                .await
        }
    })
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
