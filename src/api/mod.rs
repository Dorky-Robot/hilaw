mod edits;
mod export;
mod images;
mod preview;
mod upload;

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(upload::router())
        .merge(images::router())
        .merge(preview::router())
        .merge(edits::router())
        .merge(export::router())
}
