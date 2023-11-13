use async_openai::types::ChatCompletionResponseMessage;
use eyre::Result;

#[derive(Clone, Debug)]
pub enum Message {
    Generic(String),
    Master { content: String },
    Player { player: usize, content: String },
}
impl Message {
    pub fn as_player(self, player: usize) -> Message {
        match self {
            Message::Generic(content) => Message::Player { player, content },
            Message::Master { content } => Message::Player { player, content },
            p => p,
        }
    }
}
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Generic(content) => write!(f, "generic: {content}"),
            Message::Master { content } => write!(f, "master: {content}"),
            Message::Player { player, content } => write!(f, "player {player}: {content}"),
        }
    }
}
impl TryFrom<ChatCompletionResponseMessage> for Message {
    type Error = eyre::Error;
    fn try_from(msg: ChatCompletionResponseMessage) -> Result<Self> {
        Ok(Self::Master {
            content: msg
                .content
                .ok_or_else(|| eyre::eyre!("Couldn't convert message"))?,
        })
    }
}
