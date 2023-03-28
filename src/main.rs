mod db;
mod model;
mod views;

use crate::model::{
  TableName,
  League,
  Team,
  Division,
  TeamPlayer,
  Player,
  Shot,
  Game,
};
use views::{
  get_score_from_game,
  get_box_score_from_game,
};

use sqlx::{
  Postgres,
  Pool,
};
use axum::{
  Router,
  http::StatusCode,
  extract::{
    Path,
    State,
  },
  response::{
    Json,
    Html,
    IntoResponse,
  },
  routing::get,
};
use axum_macros::debug_handler;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerState {
  db_pool: Arc<Pool<Postgres>>,
}

#[tokio::main]
async fn main() {
  let pool = db::connect().await;
  let state = ServerState {
    db_pool: Arc::new(pool),
  }; 
  let router = Router::new()
    .route("/", get(league_html))
    .route("/league/:id/divisions/", get(divisions_for_league_html))
    .route("/division/:id/", get(games_for_division_html))
    .route("/game/:id/", get(score_for_game_html))
    .with_state(state);
  let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
  println!("Listening on {}", addr);
  axum::Server::bind(&addr)
    .serve(router.into_make_service())
    .await
    .unwrap();
}

async fn get_all<T: Send + Unpin + TableName + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>>(pool: &sqlx::PgPool) -> Result<Vec<T>, sqlx::Error> {
  sqlx::query_as::<_, T>(
    &format!("SELECT * FROM {};", <T as TableName>::TABLE_NAME)
  )
  .fetch_all(pool)
  .await
}
async fn get_by_id<T: Send + Unpin + TableName + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>>(pool: &sqlx::PgPool, id: i32) -> Result<Option<T>, sqlx::Error> {
  sqlx::query_as::<_, T>(
    &format!("SELECT * FROM {} WHERE id = $1;", <T as TableName>::TABLE_NAME)
  )
  .bind(id)
  .fetch_optional(pool)
  .await
}
/*
async fn insert_into<T: Sync + Send + Unpin + TableName + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>>(pool: &sqlx::PgPool, new: &T) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
  let query = sql_builder::SqlBuilder::insert_into(<T as TableName>::TABLE_NAME)
    .values(())
    .sql().unwrap();
  sqlx::query(
    &query
  )
  .execute(pool)
  .await
}
*/

macro_rules! get_all {
  ($crud_struct:ident, $func_name:ident) => {
    #[debug_handler]
    async fn $func_name(State(server_config): State<ServerState>) -> impl IntoResponse {
      let cruder = get_all::<$crud_struct>(&server_config.db_pool)
        .await
        .unwrap();
      (StatusCode::OK, Json(cruder))
    }
  }
}
macro_rules! get_by_id {
  ($crud_struct:ident, $func_name:ident) => {
    #[debug_handler]
    async fn $func_name(State(server_config): State<ServerState>, Path(id): Path<i32>) -> impl IntoResponse {
      let cruder = get_by_id::<$crud_struct>(&server_config.db_pool, id)
        .await
        .unwrap();
      (StatusCode::OK, Json(cruder))
    }
  }
}

async fn league_html(State(server_config): State<ServerState>) -> impl IntoResponse {
  let leagues_html = get_all::<League>(&server_config.db_pool).await
    .unwrap()
    .iter()
    .map(|league| {
      format!(
        "<li><a href=\"{1}\">{0}</a></li>",
        league.name,
        format!("/league/{}/divisions/", league.id),
      )
    })
    .collect::<Vec<String>>()
    .join("\n");
  let html = format!("<ul>{leagues_html}</ul>");
  (StatusCode::OK, Html(html))
}

async fn divisions_for_league_html(State(server_config): State<ServerState>, Path(league_id): Path<i32>) -> impl IntoResponse {
  let leagues_html = sqlx::query_as::<_, Division>("SELECT * FROM divisions WHERE league = $1")
    .bind(league_id)
    .fetch_all(&*server_config.db_pool)
    .await
    .unwrap()
    .iter()
    .map(|division| {
      format!(
        "<li><a href=\"{1}\">{0}</a></li>",
        division.name,
        format!("/division/{}/", division.id),
      )
    })
    .collect::<Vec<String>>()
    .join("\n");
  let html = format!("<ul>{leagues_html}</ul>");
  (StatusCode::OK, Html(html))
}

async fn games_for_division_html(State(server_config): State<ServerState>, Path(division_id): Path<i32>) -> impl IntoResponse {
  let leagues_html = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE division = $1")
    .bind(division_id)
    .fetch_all(&*server_config.db_pool)
    .await
    .unwrap()
    .iter()
    .map(|game| {
      format!(
        "<li><a href=\"{1}\">{0}</a></li>",
        game.name,
        format!("/game/{}/", game.id),
      )
    })
    .collect::<Vec<String>>()
    .join("\n");
  let html = format!("<ul>{leagues_html}</ul>");
  (StatusCode::OK, Html(html))
}
async fn score_for_game_html(State(server_config): State<ServerState>, Path(game_id): Path<i32>) -> impl IntoResponse {
  let game = sqlx::query_as::<_, Game>(
    "SELECT * FROM games WHERE id = $1;"
  )
  .bind(game_id)
  .fetch_one(&*server_config.db_pool)
  .await
  .unwrap();
  let score = get_score_from_game(&server_config.db_pool, &game).await.unwrap();
  let box_score_html = get_box_score_from_game(&server_config.db_pool, &game).await.unwrap()
    .iter()
    .map(|player_stats| {
      format!("<tr><td>{0}</td><td>{1}</td><td>{2}</td><td>{3}</td></tr>", player_stats.player_name, player_stats.points, player_stats.goals, player_stats.assists)
    })
    .collect::<Vec<String>>()
    .join("");
  let html = format!("<p>{}: {}<br>{}: {}</p><table>{}</table>", score.home_name, score.home, score.away_name, score.away, box_score_html);
  (StatusCode::OK, Html(html))
}

/*
macro_rules! insert {
  ($crud_struct:ident, $func_name:ident) => {
    #[debug_handler]
    async fn $func_name(State(server_config): State<ServerState>, Json(NewPlayer): Json<NewPlayer>) -> impl IntoResponse {
      let cruder = get_by_id::<$crud_struct>(&server_config.db_pool, id)
        .await
        .unwrap();
      (StatusCode::OK, Json(cruder))
    }
  }
}

macro_rules! impl_all_query_types {
  ($ty:ident, $func_all:ident, $func_by_id:ident) => {
    get_all!($ty, $func_all);
    get_by_id!($ty, $func_by_id);
  }
}

impl_all_query_types!(
  TeamPlayer,
  team_player_all,
  team_player_id
);
impl_all_query_types!(
  Player,
  player_all,
  player_id
);
impl_all_query_types!(
  Team,
  team_all,
  team_id
);
impl_all_query_types!(
  Shot,
  shots_all,
  shots_id
);
impl_all_query_types!(
  Division,
  division_all,
  division_id
);
impl_all_query_types!(
  League,
  league_all,
  league_id
);

*/
