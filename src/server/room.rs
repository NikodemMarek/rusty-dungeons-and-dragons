use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::game::{self, message::Message, Game};

#[derive(Debug)]
pub struct Room {
    pub name: String,
    pub game: Arc<Mutex<Game>>,

    pub tx: tokio::sync::broadcast::Sender<Message>,
    recv_task: tokio::task::JoinHandle<()>,

    pub players: Mutex<HashSet<usize>>,
}
impl Room {
    pub fn new(settings: game::settings::Settings, name: &str) -> Self {
        let game = Arc::new(Mutex::new(Game::new(settings)));
        let (tx, mut rx) = tokio::sync::broadcast::channel::<Message>(10);

        let recv_game = game.clone();
        let recv_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                println!("{msg}");

                let mut game = recv_game.lock().await;
                game.push_msg(msg);
            }
        });

        let players = Mutex::new(HashSet::new());

        Self {
            name: name.to_owned(),
            game,
            tx,
            recv_task,
            players,
        }
    }

    pub async fn connect(&self, character_id: usize) -> usize {
        self.players.lock().await.insert(character_id);
        character_id
    }
    pub async fn disconnect(&self, client_id: usize) {
        self.players.lock().await.remove(&client_id);
        // self.broadcast(Message::Generic(format!("disconnected: {client_id}")));
    }

    pub fn broadcast(&self, msg: Message) {
        if let Err(error) = self.tx.send(msg) {
            println!("there was an error while broadcasting a message\n{error}\naborting and closing the connection");
            let _ = self
                .tx
                .send(Message::Generic("closing due to error".to_owned())); // TODO: Send closing message
            self.recv_task.abort();
        }
    }
}
/// Make sure the reciever task is ended when the room is dropped
impl Drop for Room {
    fn drop(&mut self) {
        self.recv_task.abort();
    }
}
