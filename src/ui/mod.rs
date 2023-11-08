use askama::Template;
use axum::{
    response::Html,
    routing::{get, post},
    Router,
};

mod lobby;

use crate::types::MutState;

/// Render a template or return an error message
pub fn render_or_else<T: Template>(template: T, err: &str) -> String {
    template.render().unwrap_or_else(|_| err.to_owned())
}

#[derive(Template)]
#[template(path = "../templates/page.html")]
pub struct Page<'a> {
    title: &'a str,
    content: &'a str,
}
pub fn page(title: &str, content: &str) -> Html<String> {
    Html(render_or_else(
        Page { title, content },
        "Couldn't render page",
    ))
}

pub fn pages_router() -> Router<MutState> {
    Router::new()
        .route("/", get(index()))
        .route("/room/:room_id", get(lobby::room))
}

fn index() -> Html<String> {
    page("RDND", "<div hx-get='/c/rooms' hx-trigger='load'></div>")
}

pub fn components_router() -> Router<MutState> {
    Router::new()
        .route("/rooms", get(lobby::get_rooms))
        .route("/rooms", post(lobby::post_rooms))
}
