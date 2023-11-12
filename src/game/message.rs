use async_openai::types::ChatCompletionResponseMessage;
use eyre::Result;

#[derive(Clone, Debug)]
pub enum Message {
    Master { content: String },
    Player { player: usize, content: String },
}
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
