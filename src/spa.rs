use axum::{
    http::{StatusCode, Uri, header},
    response::{Html, IntoResponse, Response},
};
use rust_embed::Embed;

static INDEX_HTML: &str = "index.html";

#[derive(Embed)]
#[folder = "frontend/dist"]
struct Assets;

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == INDEX_HTML {
        return index_html().await;
    }

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => not_found().await,
    }
}

pub async fn index_html() -> Response {
    match Assets::get(INDEX_HTML) {
        Some(content) => Html(content.data).into_response(),
        None => not_found().await,
    }
}

async fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "").into_response()
}
