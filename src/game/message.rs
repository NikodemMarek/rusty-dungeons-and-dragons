use async_openai::types::ChatCompletionResponseMessage;
use eyre::Result;

#[derive(Clone, Debug)]
pub enum Message {
    Master(MasterMessage),
    Player(PlayerMessage),
}
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Master(m) => write!(f, "{}", m),
            Message::Player(m) => write!(f, "{}", m),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MasterMessage {
    pub content: String,
}
impl TryFrom<ChatCompletionResponseMessage> for MasterMessage {
    type Error = eyre::Error;
    fn try_from(value: ChatCompletionResponseMessage) -> Result<Self> {
        Ok(Self {
            content: value
                .content
                .ok_or_else(|| eyre::eyre!("No content"))?
                .to_owned(),
        })
    }
}
impl std::fmt::Display for MasterMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "system: {}", self.content)
    }
}

#[derive(Clone, Debug)]
pub struct PlayerMessage {
    player: usize,
    pub content: String,
}
impl PlayerMessage {
    pub fn new(player: usize, content: String) -> Self {
        Self { player, content }
    }
}
impl std::fmt::Display for PlayerMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "player {}: {}", self.player, self.content)
    }
}
