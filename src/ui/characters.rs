use askama::Template;
use axum::{extract::State, response::IntoResponse};

use crate::server::MutState;

#[derive(Template)]
#[template(path = "characters/character.html")]
struct Character<'a> {
    id: usize,
    name: &'a str,
    origin_story: &'a str,
    abilities: Vec<Ability<'a>>,
}
impl<'a> From<&'a crate::game::character::Character> for Character<'a> {
    fn from(character: &'a crate::game::character::Character) -> Self {
        let abilities = character
            .abilities
            .iter()
            .map(Into::into)
            .collect::<Vec<Ability<'a>>>();

        Self {
            id: 0,
            name: &character.name,
            origin_story: &character.origin_story,
            abilities,
        }
    }
}
#[derive(Template)]
#[template(path = "characters/ability.html")]
struct Ability<'a> {
    name: &'a str,
    description: &'a str,
}
impl<'a> From<&'a crate::game::character::Ability> for Ability<'a> {
    fn from(ability: &'a crate::game::character::Ability) -> Self {
        Self {
            name: &ability.name,
            description: &ability.description,
        }
    }
}
pub async fn character(State(state): State<MutState>) -> impl IntoResponse {
    super::utils::page_or(
        "RDND - character",
        || async {
            let client = async_openai::Client::new();
            let characters = crate::game::character::Character::new(client).await?;
            let character = characters.first().unwrap();
            Ok(Into::<Character>::into(character).render()?)
        },
        "could not render",
    )
    .await
}
