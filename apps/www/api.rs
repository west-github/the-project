#![cfg_attr(debug_assertions, allow(unused_imports))]
#[macro_use]
extern crate tracing;
extern crate derive_new;

mod routes;

use std::net::SocketAddr;

use axum::{extract::Request, middleware::Next, response::Response};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_file(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let addr = "127.0.0.1:3000".parse::<SocketAddr>()?;

    info!("SERVER \n{:?}", addr);
    axum::serve(TcpListener::bind(&addr).await?, routes::router()).await?;

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
