use askama::Template;
use axum::response::{IntoResponse, Response};
use eyre::Result;

/// Render a template or return an error message
pub fn render_or_else<T: Template>(template: &T, err: &str) -> String {
    template.render().unwrap_or_else(|_| err.to_owned())
}

/// Return a response or an error message
pub fn into_response_or<T: IntoResponse>(result: Result<T>, err: &str) -> Response {
    match result {
        Ok(result) => result.into_response(),
        Err(e) => {
            println!("{e}");
            err.to_owned().into_response()
        }
    }
}

/// Execute a function, catch the error, and return a response or an error message
pub async fn response_or<T, F, Fut>(f: F, err: &str) -> Response
where
    T: IntoResponse,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    into_response_or(f().await, err)
}
pub fn response_or_sync<T, F>(f: F, err: &str) -> Response
where
    T: IntoResponse,
    F: Fn() -> Result<T>,
{
    into_response_or(f(), err)
}

/// Return a rendered page or an error page
pub fn into_page_or(title: &str, content: Result<String>, err: &str) -> Response {
    response_or_sync(
        || {
            Ok(axum::response::Html(
                super::Page {
                    title,
                    content: match &content {
                        Ok(content) => content,
                        Err(e) => {
                            println!("{e}");
                            err
                        }
                    },
                }
                .render()?,
            )
            .into_response())
        },
        err,
    )
}

/// Execute a function, catch the error, and return a rendered page or an error page
pub fn page_or_sync<F>(title: &str, f: F, err: &str) -> Response
where
    F: Fn() -> Result<String>,
{
    into_page_or(title, f(), err)
}
pub async fn page_or<F, Fut>(title: &str, f: F, err: &str) -> Response
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<String>>,
{
    into_page_or(title, f().await, err)
}
