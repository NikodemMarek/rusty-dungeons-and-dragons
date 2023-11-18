use askama::Template;
use axum::{
    routing::{get, post},
    Router,
};

mod characters;
mod lobby;
mod room;
mod utils;

use crate::server::MutState;

#[derive(Template)]
#[template(path = "page.html")]
pub struct Page<'a> {
    title: &'a str,
    content: &'a str,
}

pub fn pages_router() -> Router<MutState> {
    Router::new()
        .route("/", get(index()))
        .route("/room/:room_id", get(room::room))
        .route("/room/:room_id/join", get(room::join_room))
}

pub fn index() -> axum::response::Html<String> {
    axum::response::Html(
        Page {
            title: "RDND",
            content: "<div hx-get='/c/rooms' hx-trigger='load'></div>",
        }
        .render()
        .unwrap_or_else(|_| "could not render page".to_owned()),
    )
}

pub fn components_router() -> Router<MutState> {
    Router::new()
        .route("/rooms", get(lobby::get_rooms))
        .route("/rooms", post(lobby::post_rooms))
        .route("/characters", get(characters::character))
}
