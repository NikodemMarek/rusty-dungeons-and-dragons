use axum::extract::ws;
use eyre::Result;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::game::Game;

#[derive(Debug)]
pub struct Client {
    pub agent: String,
    pub socket: ws::WebSocket,
    // reciever: futures_util::stream::SplitStream<ws::WebSocket>,
    // sender: futures_util::stream::SplitSink<ws::WebSocket, ws::Message>,
}
impl Client {
    pub fn new(socket: ws::WebSocket, agent: String) -> Self {
        use futures_util::StreamExt;

        // let (sender, reciever) = socket.split();
        Self {
            agent,
            socket, // reciever,
                    // sender,
        }
    }

    // pub async fn send(&mut self, msg: ws::Message) -> Result<()> {
    //     use futures_util::SinkExt;
    //     Ok(self.sender.send(msg).await?)
    // }
    // pub async fn next(&mut self) -> Result<ws::Message> {
    //     use futures_util::StreamExt;
    //     match self.reciever.next().await {
    //         Some(Ok(msg)) => Ok(msg),
    //         Some(Err(e)) => Err(e.into()),
    //         None => Err(eyre::eyre!("no message")),
    //     }
    // }
}

#[derive(Debug)]
pub struct Room {
    pub name: String,

    pub clients: std::collections::HashMap<usize, Client>,
    pub tx: tokio::sync::broadcast::Sender<String>,
    rx: tokio::sync::broadcast::Receiver<String>,
    pub game: Game,
}
impl Room {
    pub fn new(name: &str) -> Self {
        let broadcast = tokio::sync::broadcast::channel(10);

        Self {
            name: name.to_owned(),
            clients: std::collections::HashMap::new(),
            tx: broadcast.0,
            rx: broadcast.1,
            game: Game::new(),
        }
    }

    pub async fn add_client(&mut self, socket: ws::WebSocket, client: &str) {
        let id = self.game.add_player(client.to_owned());
        // self.clients.insert(id, client);

        // if let Some(client) = self.clients.get_mut(&id) {
        use futures_util::{SinkExt, StreamExt};
        let (mut sender, mut reciever) = socket.split();

        println!("atnehu");

        let mut rx = self.tx.subscribe();
        let mut send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                println!("{msg}");
                if sender.send(ws::Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });

        println!("aaaaaa");

        let tx = self.tx.clone();
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
        // }
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

    pub fn add_room(&mut self, name: &str) -> usize {
        self.rooms.insert(self.id, Room::new(name));
        self.id += 1;
        self.id
    }
}

pub type MutState = std::sync::Arc<tokio::sync::Mutex<AppState>>;
