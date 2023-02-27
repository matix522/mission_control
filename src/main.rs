#[macro_use]
extern crate lazy_static;

use anyhow::Context;
use axum::Extension;
use tokio::sync::Mutex;
use std::{net::SocketAddr, sync::Arc};
// use diesel::{PgConnection, Connection};
use diesel_async::{AsyncConnection, AsyncPgConnection};
mod handlers;
pub(crate) mod schema;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // let pool = PgPool::connect(DATABASE_URL).await?;
    dotenvy::dotenv().context(".env file missing")?;

    let database_url = std::env::var("DATABASE_URL")?;

    let connection = Arc::new(Mutex::new(AsyncPgConnection::establish(&database_url).await?));
    // build our application with a route
    let app = handlers::register().layer(Extension(connection));
    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::try_bind(&addr)?
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
