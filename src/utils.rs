use std::ops::DerefMut;

use axum::http::StatusCode;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::bb8::Pool;

pub(crate) async fn get_db_connection<'a>(
    pool: &'a Pool<AsyncPgConnection>,
) -> Result<impl DerefMut<Target = AsyncPgConnection> + 'a, StatusCode> {
    pool.get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
