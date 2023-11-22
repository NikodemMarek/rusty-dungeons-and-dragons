use askama::Template;

use axum::{
    extract::{connect_info::ConnectInfo, ws, Path, State},
    response::IntoResponse,
    Form, TypedHeader,
};
use eyre::Result;
use std::net::SocketAddr;

use crate::{
    game,
    server::{self, MutState},
};

use super::utils::{page_or, render_or_else, response_or};

#[derive(Template)]
#[template(path = "room/page.html")]
struct Page<'a> {
    id: usize,
    name: &'a str,
    messages: &'a [Message<'a>],
}
pub async fn room(State(state): State<MutState>, Path(room_id): Path<usize>) -> impl IntoResponse {
    let rs = &mut state.lock().await;

    page_or(
        "RDND - room",
        || async {
            let room = rs.get_room(&room_id)?;
            let content = Page {
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
            }
            .render()?;
            Ok(content)
        },
        "could not render room",
    )
    .await
}

#[derive(Template)]
#[template(path = "room/ws.html")]
struct Ws {
    room_id: usize,
    character_id: usize,
}
#[derive(Debug, serde::Deserialize)]
pub struct NewPlayer {
    pub character_id: usize,
}
pub async fn join(
    State(state): State<MutState>,
    Path(room_id): Path<usize>,
    Form(new_player): Form<NewPlayer>,
) -> impl IntoResponse {
    let rs = &mut state.lock().await;

    response_or(
        || async {
            let room = rs.get_room(&room_id)?;

            // TODO: Validate if character is avaliable or sth

            let content = Ws {
                room_id,
                character_id: new_player.character_id,
            }
            .render()?;
            Ok(content)
        },
        "could not join room",
    )
    .await
}

pub async fn ws(
    State(state): State<MutState>,
    Path((room_id, character_id)): Path<(usize, usize)>,

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
        let client_id = room.connect(character_id).await;

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
                render_or_else(&Message::from(&msg), "could not render message").into(),
            );

            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let tx = room.tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(ws::Message::Text(text))) = reciever.next().await {
            if let Ok(msg) = TryInto::<game::message::Message>::try_into(text) {
                let _ = tx.send(msg.as_player(client_id));
            }
        }
    });

    println!("client {agent} connected with id {client_id}");

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    room.disconnect(client_id).await;
    println!("client {agent} disconnected");
}

#[derive(Template)]
#[template(path = "room/message.html")]
struct Message<'a> {
    content: &'a str,
}
#[derive(Debug, serde::Deserialize)]
struct RawMessage {
    message: String,
}
impl TryFrom<&str> for RawMessage {
    type Error = eyre::Error;
    fn try_from(msg: &str) -> Result<Self> {
        Ok(serde_json::from_str(msg)?)
    }
}
impl Into<game::message::Message> for RawMessage {
    fn into(self) -> game::message::Message {
        game::message::Message::Generic(self.message)
    }
}
impl TryFrom<String> for game::message::Message {
    type Error = eyre::Error;
    fn try_from(msg: String) -> Result<Self> {
        Ok(TryInto::<RawMessage>::try_into(msg.as_str())?.into())
    }
}
impl<'a> From<&'a game::message::Message> for Message<'a> {
    fn from(msg: &'a game::message::Message) -> Self {
        match msg {
            game::message::Message::Generic(content) => Self { content },
            game::message::Message::Master { content } => Self { content },
            game::message::Message::Player { player: _, content } => Self { content },
        }
    }
}
