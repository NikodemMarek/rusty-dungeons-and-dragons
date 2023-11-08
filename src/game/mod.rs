use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionResponseMessage,
        CreateChatCompletionRequestArgs, Role,
    },
    Client,
};
use eyre::Result;

#[derive(Debug)]
struct ContextMessage {
    role: Role,
    content: String,
}
impl Into<ChatCompletionRequestMessage> for &ContextMessage {
    fn into(self) -> ChatCompletionRequestMessage {
        ChatCompletionRequestMessage {
            role: self.role,
            content: Some(self.content.to_owned()),
            ..ChatCompletionRequestMessage::default()
        }
    }
}
impl TryFrom<ChatCompletionResponseMessage> for ContextMessage {
    type Error = eyre::Error;
    fn try_from(value: ChatCompletionResponseMessage) -> Result<Self> {
        Ok(Self {
            role: value.role,
            content: value
                .content
                .ok_or_else(|| eyre::eyre!("No content"))?
                .to_owned(),
        })
    }
}

#[derive(Debug)]
pub struct Game {
    client: Client<OpenAIConfig>,
    context: Vec<ContextMessage>,
}
impl Game {
    pub fn new() -> Self {
        let client = Client::new();
        let context = Vec::from([ContextMessage {
            role: Role::System,
            content: String::from("You are a dnd game master"),
        }]);

        Self { client, context }
    }

    fn push(&mut self, msg: ContextMessage) {
        self.context.push(msg)
    }

    pub async fn next(&mut self) -> Result<()> {
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .messages(
                self.context
                    .iter()
                    .map(|m| m.into())
                    .collect::<Vec<ChatCompletionRequestMessage>>(),
            )
            .build()?;

        println!("oetaush");
        let response = match self.client.chat().create(request).await {
            Ok(response) => response,
            Err(e) => {
                println!("Error: {}", e);
                return Ok(());
            }
        };

        let rm = response.choices.first();
        rm.map(|rm| {
            if let Ok(m) = rm.message.to_owned().try_into() {
                self.push(m);
            }
        });

        println!("\nResponse:\n");
        for choice in response.choices {
            println!(
                "{}: Role: {}  Content: {:?}",
                choice.index, choice.message.role, choice.message.content
            );
        }

        Ok(())
    }

    pub fn messages(&self) -> Vec<String> {
        self.context.iter().map(|m| m.content.to_owned()).collect()
    }
}
