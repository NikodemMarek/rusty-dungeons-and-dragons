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
    if let Some(_) = state.lock().await.rooms.get(&room_id) {
        ws.on_upgrade(|socket| handle_socket(socket, state))
    } else {
        ws.on_upgrade(|_| async {})
        // axum::response::Response::<axum::body::Body>::new(axum::body::Body::from("Room not found"))
    }
}

async fn handle_socket(socket: ws::WebSocket, state: MutState) {
    if let Some(room) = state.lock().await.rooms.get(&0) {
        use futures_util::{SinkExt, StreamExt};
        let (mut sender, mut reciever) = socket.split();

        println!("atnehu");

        let mut rx = room.tx.subscribe();
        let mut send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                println!("{msg}");
                if sender.send(ws::Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });

        println!("aaaaaa");

        let tx = room.tx.clone();
        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(ws::Message::Text(text))) = reciever.next().await {
                println!("{text}");
                let _ = tx.send(format!(": {text}"));
            }
        });

        println!("duuuuuu");

        tokio::select! {
            _ = (&mut send_task) => recv_task.abort(),
            _ = (&mut recv_task) => send_task.abort(),
        };

        println!("Websocket context destroyed");
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
