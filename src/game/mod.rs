use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, Role},
    Client,
};
use eyre::Result;
use std::collections::HashMap;

pub mod message;

use message::Message;

#[derive(Debug)]
pub struct Player {
    name: String,
}

#[derive(Debug)]
struct Context {
    config: String,
    players: Vec<String>,
    location: String,
    story: Vec<String>,
}
impl From<Game> for Context {
    fn from(game: Game) -> Self {
        Self {
            config: String::from(""),
            players: game
                .players
                .iter()
                .map(|p| p.1.name.to_owned())
                .collect::<Vec<String>>(),
            location: String::from(""),
            story: game
                .messages
                .iter()
                .map(|m| {
                    match m {
                        Message::Master { content } => content,
                        Message::Player { player, content } => content,
                    }
                    .to_owned()
                })
                .collect::<Vec<String>>(),
        }
    }
}
impl TryInto<Vec<ChatCompletionRequestMessage>> for &Context {
    type Error = eyre::Error;
    fn try_into(self) -> Result<Vec<ChatCompletionRequestMessage>> {
        use async_openai::types::ChatCompletionRequestMessageArgs;

        if let (Ok(config), Ok(players), Ok(location), Ok(story)) = (
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(self.config.clone())
                .build(),
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(self.players.join("\n"))
                .build(),
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(self.location.clone())
                .build(),
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(self.story.join("\n"))
                .build(),
        ) {
            Ok(vec![config, players, location, story])
        } else {
            Err(eyre::eyre!("Couldn't build context"))
        }
    }
}

#[derive(Debug)]
pub struct Game {
    client: Client<OpenAIConfig>,
    context: Context,
    players: HashMap<usize, Player>,
    messages: Vec<Message>,
}
impl Game {
    pub fn new() -> Self {
        let client = Client::new();
        let context = Context {
            config: String::from(""),
            players: Vec::new(),
            location: String::from(""),
            story: Vec::new(),
        };
        let players = HashMap::new();
        let messages = Vec::new();

        Self {
            client,
            context,
            players,
            messages,
        }
    }

    pub fn add_player(&mut self, name: String) -> usize {
        let id = self.players.len();
        self.players.insert(id, Player { name });
        id
    }

    pub fn push_msg(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    pub async fn next(&mut self) -> Result<()> {
        let msgs: Vec<ChatCompletionRequestMessage> = (&self.context).try_into()?;
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .messages(msgs)
            .build()?;

        let response = match self.client.chat().create(request).await {
            Ok(response) => response,
            Err(e) => {
                println!("Error: {}", e);
                return Ok(());
            }
        };

        response.choices.first().map(|rm| {
            if let Ok(msg) = rm.message.to_owned().try_into() {
                self.push_msg(msg)
            }
        });

        Ok(())
    }

    pub fn messages<'a>(&'a self) -> &'a [Message] {
        &self.messages
    }
}
