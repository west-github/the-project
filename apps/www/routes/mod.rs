use axum::{
    extract::Request,
    middleware::{from_fn, Next},
    response::Response,
    routing::get,
    Router,
};

use crate::debug_logger;

pub async fn handler() -> &'static str {
    info!("GET ---> HANDLER");

    "We got Something"
}

pub fn router() -> Router<()> {
    Router::<()>::new()
        .route("/", get(handler))
        .layer(from_fn(debug_logger))
}
