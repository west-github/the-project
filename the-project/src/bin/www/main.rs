#![cfg_attr(debug_assertions, allow(unused_imports, dead_code))]
#[macro_use]
extern crate the_project;
extern crate tracing;

use axum::{
    extract::Request,
    middleware::{from_fn, Next},
    response::Response,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use the_project::Static;
use tokio::net::TcpListener;
use tracing::info;

#[derive(new, Debug)]
pub struct Data(&'static str);

impl Clone for Data {
    fn clone(&self) -> Self {
        println!("Data cloned");
        Self(self.0)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .pretty()
        .with_file(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_max_level(tracing::Level::DEBUG)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .init();

    let addr = "127.0.0.1:3000".parse::<SocketAddr>()?;
    let data: &'static Data = to_static!(Data, Data::new("Data"));

    let app = Router::<()>::new()
        .route("/", get(handler))
        .layer(from_fn(debug_logger))
        .layer(static_s!(data));

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

pub async fn handler(Static(data): Static<Data>) -> &'static str {
    println!("{:?} with content {:?}", data, data.0);

    info!("GET ---> HANDLER");
    "We got Something"
}
