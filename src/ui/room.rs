use askama::Template;

use axum::{
    extract::{connect_info::ConnectInfo, ws, Path, State},
    response::{Html, IntoResponse},
    TypedHeader,
};
use std::net::SocketAddr;

use crate::{
    game,
    server::{self, MutState},
    ui::page,
};

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
                        .lock()
                        .await
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
    if let Some(room) = state
        .lock()
        .await
        .rooms
        .get(&room_id)
        .map(|room| room.clone())
    {
        let agent = format!(
            "[{addr}] {}",
            if let Some(TypedHeader(user_agent)) = user_agent {
                user_agent.to_string()
            } else {
                String::from("Unknown browser")
            }
        );
        let client_id = room.add_client(&agent).await;

        ws.on_upgrade(move |socket| handle_socket(socket, room, client_id, agent))
    } else {
        ws.on_upgrade(|_| async { () })
    }
}

async fn handle_socket(
    socket: ws::WebSocket,
    room: std::sync::Arc<server::room::Room>,
    client_id: usize,
    agent: String,
) {
    use futures_util::{SinkExt, StreamExt};
    let (mut sender, mut reciever) = socket.split();

    let mut rx = room.tx.subscribe();
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let msg = ws::Message::Text(
                render_or_else(Message::from(&msg), "Couldn't render message").into(),
            );

            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let tx = room.tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(ws::Message::Text(content))) = reciever.next().await {
            // let msg = Player(PlayerMessage::new(client_id, text));
            let msg = game::message::Message::Player {
                player: client_id,
                content,
            };
            let _ = tx.send(msg);
        }
    });

    println!("client {agent} connected with id {client_id}");

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    println!("client {agent} disconnected");
}

#[derive(Template)]
#[template(path = "room/message.html")]
struct Message<'a> {
    content: &'a str,
}
impl<'a> From<&'a game::message::Message> for Message<'a> {
    fn from(msg: &'a game::message::Message) -> Self {
        match msg {
            game::message::Message::Master { content } => Self { content },
            game::message::Message::Player { player, content } => Self { content },
        }
    }
}
