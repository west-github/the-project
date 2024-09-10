#![cfg_attr(debug_assertions, allow(unused_imports))]
#[macro_use]
extern crate tracing;
extern crate the_project;

use axum::{
    extract::Request,
    middleware::{from_fn, Next},
    response::Response,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .pretty()
        .with_file(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let addr = "127.0.0.1:3000".parse::<SocketAddr>()?;

    let app = Router::<()>::new()
        .route("/", get(handler))
        .layer(from_fn(debug_logger));

    info!("SERVER \n{:?}", addr);
    axum::serve(TcpListener::bind(&addr).await?, app).await?;

    Ok(())
}

pub async fn debug_logger(req: Request, next: Next) -> Response {
    #[cfg(debug_assertions)]
    {
        let (parts, body) = req.into_parts();

        info!("Got a request with parts: {:#?}", parts);
        next.run(Request::from_parts(parts, body)).await
    }

    #[cfg(not(debug_assertions))]
    {
        next.run(req).await
    }
}

pub async fn handler() -> &'static str {
    info!("GET ---> HANDLER");

    "We got Something"
}
