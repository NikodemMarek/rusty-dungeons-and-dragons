use askama::Template;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

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
pub async fn character(
    State(state): State<MutState>,
    Path((room_id, character_id)): Path<(usize, usize)>,
) -> impl IntoResponse {
    let rs = &mut state.lock().await;

    super::utils::response_or(
        || async {
            let room = rs.get_room(&room_id)?;
            let game = room.game.lock().await;

            let character = game.get_character(&character_id)?;

            let content = Into::<Character>::into(character).render()?;
            Ok(content)
        },
        "could not render",
    )
    .await
}
pub async fn characters(
    State(state): State<MutState>,
    Path(room_id): Path<usize>,
) -> impl IntoResponse {
    let rs = &mut state.lock().await;

    super::utils::response_or(
        || async {
            let room = rs.get_room(&room_id)?;
            let game = room.game.lock().await;

            let content = game
                .get_characters()
                .values()
                .map(Into::<Character>::into)
                .map(|c| c.render())
                .collect::<Result<Vec<String>, askama::Error>>()?
                .join("<br>");
            Ok(content)
        },
        "could not render",
    )
    .await
}
