use eyre::Result;

#[derive(Debug, serde::Deserialize)]
pub struct Character {
    pub name: String,
    pub origin_story: String,
    pub abilities: Vec<Ability>,
}
impl Character {
    pub async fn new(
        client: async_openai::Client<async_openai::config::OpenAIConfig>,
    ) -> Result<Vec<Character>> {
        let request = async_openai::types::CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo-1106")
            .messages([
                async_openai::types::ChatCompletionRequestSystemMessageArgs::default()
                    .content(String::from(
                        r#"you are a dungeons and dragons character creator
on [create] create a balanced character
character needs to have:
- name
- origin_story - 1 paragraph
- abilities - between 2 to 4, consisting of name and description
return the character as JSON"#,
                    ))
                    .build()?
                    .into(),
                async_openai::types::ChatCompletionRequestUserMessageArgs::default()
                    .content(String::from("[create]"))
                    .build()?
                    .into(),
            ])
            .response_format(async_openai::types::ChatCompletionResponseFormat {
                r#type: async_openai::types::ChatCompletionResponseFormatType::JsonObject,
            })
            .n(1)
            .build()?;

        let response = match client.chat().create(request).await {
            Ok(response) => response,
            Err(_) => return Err(eyre::eyre!("could not create a character")),
        };

        let characters = response
            .choices
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Character>>>()?;

        Ok(characters)
    }
}
impl TryFrom<&str> for Character {
    type Error = eyre::Error;
    fn try_from(msg: &str) -> Result<Self> {
        Ok(serde_json::from_str(msg)?)
    }
}
impl TryFrom<async_openai::types::ChatChoice> for Character {
    type Error = eyre::Error;
    fn try_from(msg: async_openai::types::ChatChoice) -> Result<Self> {
        let content = match msg.message.content {
            Some(content) => content,
            None => return Err(eyre::eyre!("could not convert chat message to character")),
        };

        Ok(Self::try_from(content.as_str())?)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Ability {
    pub name: String,
    pub description: String,
}
