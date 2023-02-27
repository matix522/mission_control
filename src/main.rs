use anyhow::Context;
use axum::Extension;
use diesel_async::pooled_connection::{bb8::Pool, AsyncDieselConnectionManager};
use diesel_async::AsyncPgConnection;
use std::net::SocketAddr;
mod handlers;
pub(crate) mod schema;
pub(crate) mod utils;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // let pool = PgPool::connect(DATABASE_URL).await?;
    dotenvy::dotenv().context(".env file missing")?;

    let database_url = std::env::var("DATABASE_URL")?;

    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder()
        .test_on_check_out(true)
        .max_size(20)
        .build(manager)
        .await?;

    // build our application with a route
    let app = handlers::register().layer(Extension(pool));
    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::try_bind(&addr)?
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
