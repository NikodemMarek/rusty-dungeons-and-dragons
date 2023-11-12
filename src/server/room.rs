use std::sync::Arc;
use tokio::sync::Mutex;

use crate::game::Game;

#[derive(Debug)]
pub struct Room {
    pub name: String,
    pub game: Arc<Mutex<Game>>,

    pub tx: tokio::sync::broadcast::Sender<String>,
    recv_task: tokio::task::JoinHandle<()>,
}
impl Room {
    pub fn new(name: &str) -> Self {
        let game = Arc::new(Mutex::new(Game::new()));
        let (tx, mut rx) = tokio::sync::broadcast::channel::<String>(10);

        let recv_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                // TODO: Somehow get the sender from the message
                let mut game = game.lock().await;
                game.push_player_msg(0, msg.clone());

                println!("sbd said: {msg}");
            }
        });

        Self {
            name: name.to_owned(),
            game: Arc::new(Mutex::new(Game::new())),
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
