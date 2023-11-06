use axum::routing::post;
use axum::{routing::get, Router};

use crate::types::MutState;

/// Render a template or return an error message
fn render_or_else<T: askama::Template>(template: T, err: &str) -> String {
    template.render().unwrap_or_else(|_| err.to_owned())
}

pub fn pages_router() -> Router<MutState> {
    Router::new()
        .route("/", get(pages::index()))
        .route("/room/:room_id", get(pages::room))
}

mod pages {
    use askama::Template;
    use axum::extract::{Path, State};
    use axum::response::Html;

    use crate::types::MutState;

    use super::render_or_else;

    #[derive(Template)]
    #[template(path = "../templates/page.html")]
    struct Page<'a> {
        title: &'a str,
        content: &'a str,
    }

    fn page(title: &str, content: &str) -> Html<String> {
        Html(render_or_else(
            Page { title, content },
            "Couldn't render page",
        ))
    }

    pub fn index() -> Html<String> {
        page("RDND", "<div hx-get='/c/rooms' hx-trigger='load'></div>")
    }

    pub async fn room(Path(room_id): Path<usize>, State(state): State<MutState>) -> Html<String> {
        match state.lock() {
            Ok(rs) => match rs.rooms.get(&room_id) {
                Some(room) => page("RDND - room", format!("Room: {}", room.name).as_str()),
                None => page("RDND - room", "Room not found"),
            },
            Err(_) => page("RDND - room", "Room not found"),
        }
    }
}

pub fn components_router() -> Router<MutState> {
    Router::new()
        .route("/rooms", get(components::get_rooms))
        .route("/rooms", post(components::post_rooms))
}

mod components {
    use askama::Template;
    use axum::extract::State;
    use axum::Form;

    use crate::types::MutState;

    use super::render_or_else;

    #[derive(Template)]
    #[template(path = "../templates/rooms/room.html")]
    struct Room<'a> {
        name: &'a str,
        id: &'a usize,
    }
    #[derive(Debug, serde::Deserialize)]
    pub struct NewRoom {
        pub name: String,
    }
    pub async fn post_rooms(
        State(state): State<MutState>,
        Form(new_room): Form<NewRoom>,
    ) -> String {
        match &mut state.lock() {
            Ok(rs) => {
                let room_id = rs.add_room(new_room.name);
                match rs.rooms.get(&room_id) {
                    Some(room) => render_or_else(
                        Room {
                            name: &room.name,
                            id: &room_id,
                        },
                        "Couldn't render room",
                    ),
                    _ => String::from("Couldn't add a room"),
                }
            }
            Err(_) => String::from("Couldn't add a room"),
        }
    }

    #[derive(Template)]
    #[template(path = "../templates/rooms/rooms.html")]
    struct Rooms<'a> {
        rooms: Vec<Room<'a>>,
    }
    pub async fn get_rooms(State(state): State<MutState>) -> String {
        match state.lock() {
            Ok(rs) => render_or_else(
                Rooms {
                    rooms: rs
                        .rooms
                        .iter()
                        .map(|r| Room {
                            name: &r.1.name,
                            id: r.0,
                        })
                        .collect::<Vec<Room>>(),
                },
                "Couldn't render room list",
            ),
            Err(_) => String::from("Couldn't load room list"),
        }
    }
}
