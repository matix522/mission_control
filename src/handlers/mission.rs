use std::{ops::DerefMut, sync::Arc};

use diesel_async::{AsyncPgConnection, RunQueryDsl};
use tokio::sync::Mutex;

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

type DbConnection = Arc<Mutex<AsyncPgConnection>>;

pub(crate) fn register() -> Router {
    Router::new()
        .route("/missions", get(get_missions))
        .route("/mission/:mission_id", get(get_mission))
        .route("/mission/:mission_id", delete(delete_mission))
        .route("/mission", post(add_mission))
}

#[derive(Queryable, Serialize, Clone, Debug)]
struct Mission {
    mission_id: i32,
    mission_name: String,
    location: String,
    tags: Vec<Option<String>>,
}
#[derive(Deserialize)]
struct AddMission {
    mission_name: String,
    location: String,
    tags: Option<Vec<String>>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::missions)]
struct NewMission {
    mission_name: String,
    location: String,
    tags: Vec<String>,
}

async fn delete_mission(
    Path(url_mission_ids): Path<i32>,
    Extension(db): Extension<DbConnection>,
) -> StatusCode {
    use crate::schema::missions::dsl::*;

    let mut db = db.lock().await;

    if let Err(e) = diesel::delete(missions.filter(mission_id.eq(url_mission_ids)))
        .execute(db.deref_mut())
        .await
    {
        dbg!(e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

async fn get_mission(
    Path(url_mission_ids): Path<i32>,
    Extension(db): Extension<DbConnection>,
) -> Result<Json<Mission>, StatusCode> {
    use crate::schema::missions::dsl::*;

    let mut db = db.lock().await;

    let results: Vec<_> = missions
        .filter(mission_id.eq(url_mission_ids))
        .limit(1)
        .load::<Mission>(db.deref_mut())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(mission) = results.last() {
        return Ok(Json(mission.clone()));
    }
    Err(StatusCode::NOT_FOUND)
}

async fn add_mission(
    Extension(db): Extension<DbConnection>,
    Json(add_mission): Json<AddMission>,
) -> (StatusCode, Json<Option<i32>>) {
    use crate::schema::missions::dsl::*;
    let mut db = db.lock().await;

    let new_mission = NewMission {
        mission_name: add_mission.mission_name,
        location: add_mission.location,
        tags: add_mission.tags.unwrap_or_default(),
    };
    let result = diesel::insert_into(missions)
        .values(&new_mission)
        .get_result::<Mission>(db.deref_mut())
        .await;

    if let Ok(mission) = dbg!(result) {
        return (StatusCode::CREATED, Json(Some(mission.mission_id)));
    }
    (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
}

async fn get_missions(
    Extension(db): Extension<DbConnection>,
) -> Result<Json<Vec<Mission>>, StatusCode> {
    tracing::debug!("Get missions request");
    use crate::schema::missions::dsl::*;
    let mut db = db.lock().await;

    let results = missions
        .load::<Mission>(db.deref_mut())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // this will be converted into a JSON response
    // with a status code of `200 OK`
    Ok(Json(results))
}
