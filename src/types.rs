use axum::extract::ws;
use eyre::Result;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::game::Game;

#[derive(Debug)]
pub struct Client {
    pub agent: String,
    reciever: futures_util::stream::SplitStream<ws::WebSocket>,
    sender: futures_util::stream::SplitSink<ws::WebSocket, ws::Message>,
}
impl Client {
    pub fn new(socket: ws::WebSocket, agent: String) -> Self {
        use futures_util::StreamExt;

        let (sender, reciever) = socket.split();
        Self {
            agent,
            reciever,
            sender,
        }
    }

    pub async fn send(&mut self, msg: ws::Message) -> Result<()> {
        use futures_util::SinkExt;
        Ok(self.sender.send(msg).await?)
    }
    pub async fn next(&mut self) -> Result<ws::Message> {
        use futures_util::StreamExt;
        match self.reciever.next().await {
            Some(Ok(msg)) => Ok(msg),
            Some(Err(e)) => Err(e.into()),
            None => Err(eyre::eyre!("no message")),
        }
    }
}

#[derive(Debug)]
pub struct Room {
    pub name: String,

    pub clients: std::collections::HashMap<usize, Arc<Mutex<Client>>>,
    tx: tokio::sync::broadcast::Sender<String>,
    rx: tokio::sync::broadcast::Receiver<String>,
    pub game: Game,
}
impl Room {
    pub fn new(name: String) -> Self {
        let broadcast = tokio::sync::broadcast::channel(10);

        Self {
            name,
            clients: std::collections::HashMap::new(),
            tx: broadcast.0,
            rx: broadcast.1,
            game: Game::new(),
        }
    }

    pub async fn add_client(&mut self, client: Client) {
        let id = self.game.add_player(client.agent.clone());
        self.clients.insert(id, Arc::new(Mutex::new(client)));

        if let Some(client) = self.clients.get(&id) {
            let recv_client = client.clone();
            let send_client = client.clone();

            let mut rx = self.tx.subscribe();
            let mut send_task = tokio::spawn(async move {
                let mut client = send_client.lock().await;

                while let Ok(msg) = rx.recv().await {
                    println!("{msg}");
                    if client.send(ws::Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
            });

            let tx = self.tx.clone();
            let mut recv_task = tokio::spawn(async move {
                let mut client = recv_client.lock().await;

                while let Ok(ws::Message::Text(text)) = client.next().await {
                    println!("{text}");
                    let _ = tx.send(format!(": {text}"));
                }
            });

            tokio::select! {
                _ = (&mut send_task) => recv_task.abort(),
                _ = (&mut recv_task) => send_task.abort(),
            };

            println!("Websocket context destroyed");
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
