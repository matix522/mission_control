use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::bb8::Pool;
use std::ops::DerefMut;

use diesel_async::RunQueryDsl;

use crate::utils;
use axum::{
    extract::Path,
    http::StatusCode,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use diesel::{prelude::*, sql_types::Bool};
use serde::{Deserialize, Serialize};

type DbConnectionPool = Pool<AsyncPgConnection>;
type DB = diesel::pg::Pg;

pub(crate) fn register() -> Router {
    Router::new()
        .route("/missions", get(get_missions))
        .route("/mission", post(add_mission))
        .route("/mission/:mission_id", get(get_mission_by_id))
        .route("/mission/:mission_id", delete(delete_mission_by_id))
        .route("/mission/name/:mission_id", get(get_mission_by_name))
        .route("/mission/name/:mission_id", delete(delete_mission_by_name))
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
type MissionMatcher<'a> = Box<dyn BoxableExpression<crate::schema::missions::table, DB, SqlType = Bool> + 'a>;

enum MatchMission<'a> {
    ById(i32),
    ByName(&'a str)
}

impl <'a> MatchMission<'a> {
    fn make_expresion(&'a self) -> MissionMatcher<'a> {
        match self {
            MatchMission::ById(id) => by_id(*id),
            MatchMission::ByName(name) => by_name(name)
        }
    }
}


fn by_name<'a>(name: &'a str) -> MissionMatcher<'a>
{
    use crate::schema::missions::dsl::*;

    Box::new(mission_name.eq(name))
}
fn by_id<'a>(id : i32) -> MissionMatcher<'a>
{
    use crate::schema::missions::dsl::*;

    Box::new(mission_id.eq(id))
}

async fn delete_mission_by_id(
    Path(url_mission_ids): Path<i32>,
    Extension(db_pool): Extension<DbConnectionPool>,
) -> Result<(), StatusCode> {
    delete_mission_by_predicate( MatchMission::ById(url_mission_ids), db_pool).await
}
async fn delete_mission_by_name(
    Path(url_mission_name): Path<String>,
    Extension(db_pool): Extension<DbConnectionPool>,
) -> Result<(), StatusCode> {
    delete_mission_by_predicate( MatchMission::ByName(&url_mission_name), db_pool).await
}

async fn delete_mission_by_predicate(
    predicate : MatchMission<'_>,
    db_pool: DbConnectionPool,
) -> Result<(), StatusCode> {
    use crate::schema::missions::dsl::*;

    let mut db = utils::get_db_connection(&db_pool).await?;
    if let Err(e) = diesel::delete(missions.filter(predicate.make_expresion()))
        .execute(db.deref_mut())
        .await
    {
        dbg!(e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Err(StatusCode::OK)
}

async fn get_mission_by_predicate(
    predicate : MatchMission<'_>,
    db_pool: DbConnectionPool,
) -> Result<Json<Mission>, StatusCode> {
    use crate::schema::missions::dsl::*;

    let mut db = utils::get_db_connection(&db_pool).await?;

    let results: Vec<_> = missions
        .filter(predicate.make_expresion())
        .limit(1)
        .load::<Mission>(db.deref_mut())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(mission) = results.last() {
        return Ok(Json(mission.clone()));
    }
    Err(StatusCode::NOT_FOUND)
}

async fn get_mission_by_id(
    Path(url_mission_ids): Path<i32>,
    Extension(db_pool): Extension<DbConnectionPool>,
) -> Result<Json<Mission>, StatusCode> {
    get_mission_by_predicate( MatchMission::ById(url_mission_ids), db_pool).await

}
async fn get_mission_by_name(
    Path(url_mission_name): Path<String>,
    Extension(db_pool): Extension<DbConnectionPool>,
) -> Result<Json<Mission>, StatusCode> {
    get_mission_by_predicate( MatchMission::ByName(&url_mission_name), db_pool).await

}

async fn add_mission(
    Extension(db_pool): Extension<DbConnectionPool>,
    Json(add_mission): Json<AddMission>,
) -> (StatusCode, Json<Option<i32>>) {
    use crate::schema::missions::dsl::*;
    let mut db = match utils::get_db_connection(&db_pool).await {
        Ok(db) => db,
        Err(e) => return (e, Json(None)),
    };

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
    Extension(db_pool): Extension<DbConnectionPool>,
) -> Result<Json<Vec<Mission>>, StatusCode> {
    tracing::debug!("Get missions request");
    use crate::schema::missions::dsl::*;
    let mut db = utils::get_db_connection(&db_pool).await?;

    let results = missions
        .load::<Mission>(db.deref_mut())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // this will be converted into a JSON response
    // with a status code of `200 OK`
    Ok(Json(results))
}
