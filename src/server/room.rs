use std::sync::Arc;
use tokio::sync::Mutex;

use crate::game::{message::Message, Game};

#[derive(Debug)]
pub struct Room {
    pub name: String,
    pub game: Arc<Mutex<Game>>,

    pub tx: tokio::sync::broadcast::Sender<Message>,
    recv_task: tokio::task::JoinHandle<()>,
}
impl Room {
    pub fn new(name: &str) -> Self {
        let game = Arc::new(Mutex::new(Game::new()));
        let (tx, mut rx) = tokio::sync::broadcast::channel::<Message>(10);

        let recv_game = game.clone();
        let recv_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                println!("{msg}");

                let mut game = recv_game.lock().await;
                game.push_msg(msg);
            }
        });

        Self {
            name: name.to_owned(),
            game,
            tx,
            recv_task,
        }
    }

    pub async fn add_client(&self, agent: &str) -> usize {
        self.game.lock().await.add_player(agent.to_owned())
    }
}
/// Make sure the reciever task is ended when the room is dropped
impl Drop for Room {
    fn drop(&mut self) {
        self.recv_task.abort();
    }
}
