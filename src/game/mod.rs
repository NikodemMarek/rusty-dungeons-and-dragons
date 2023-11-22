use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use eyre::Result;
use std::collections::HashMap;

pub mod character;
pub mod message;
pub mod settings;

use message::Message;

use self::character::{generate_characters, Character};

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
                .characters
                .iter()
                .map(|p| p.1.name.to_owned())
                .collect::<Vec<String>>(),
            location: String::from(""),
            story: game
                .messages
                .iter()
                .map(|m| {
                    match m {
                        Message::Generic(content) => content,
                        Message::Master { content } => content,
                        Message::Player { player: _, content } => content,
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
        if let (Ok(config), Ok(players), Ok(location), Ok(story)) = (
            ChatCompletionRequestSystemMessageArgs::default()
                .content(self.config.clone())
                .build(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(self.players.join("\n"))
                .build(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(self.location.clone())
                .build(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(self.story.join("\n"))
                .build(),
        ) {
            Ok(Vec::from([
                config.into(),
                players.into(),
                location.into(),
                story.into(),
            ]))
        } else {
            Err(eyre::eyre!("Couldn't build context"))
        }
    }
}

#[derive(Debug)]
pub struct Game {
    settings: settings::Settings,
    client: Client<OpenAIConfig>,
    context: Context,
    characters: HashMap<usize, Character>,
    messages: Vec<Message>,
}
impl Game {
    pub fn new(settings: settings::Settings) -> Self {
        let client = Client::new();
        let context = Context {
            config: String::from(""),
            players: Vec::new(),
            location: String::from(""),
            story: Vec::new(),
        };
        let characters = HashMap::new();
        let messages = Vec::new();

        Self {
            settings,
            client,
            context,
            characters,
            messages,
        }
    }

    pub async fn init(&mut self) -> Result<()> {
        let characters = generate_characters(&self.client, self.settings.player_count as u8);
        for (i, character) in characters.await?.into_iter().enumerate() {
            self.characters.insert(i, character);
        }

        Ok(())
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

    pub fn get_characters(&self) -> &HashMap<usize, Character> {
        &self.characters
    }
    pub fn get_character(&self, character_id: &usize) -> Result<&Character> {
        self.characters
            .get(character_id)
            .ok_or_else(|| eyre::eyre!("player with id {} not found", character_id))
    }
}
