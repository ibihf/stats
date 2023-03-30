mod db;
mod model;
mod views;
mod filters;

use crate::model::{
  League,
  Team,
  Division,
  TeamPlayer,
  Player,
  Shot,
  Game,
};
use views::{
  GoalDetails,
  PlayerStats,
  TeamStats,
  ShotDetails,
  get_score_from_game,
  get_box_score_from_game,
  get_play_by_play_from_game,
  get_goals_from_game,
  get_latest_league_for_player,
  get_league_player_stats,
  get_all_player_stats,
};

use sqlx::{
  Postgres,
  Pool,
};
use ormx::Table;
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
use askama::Template;

#[derive(Template)]
#[template(path="hello.html")]
struct HelloTemplate<'a> {
  name: &'a str,
  years: i32
}

#[derive(Template)]
#[template(path="partials/box_score_table.html")]
struct BoxScoreTemplate {
  goals: Vec<GoalDetails>,
}

#[derive(Template)]
#[template(path="partials/individual_game_points_table.html")]
struct IndividualGamePointsTableTemplate {
  players: Vec<PlayerStats>,
}

#[derive(Template)]
#[template(path="partials/team_stats_table.html")]
struct TeamGameStatsTemplate {
  teams: Vec<TeamStats>,
}

#[derive(Template)]
#[template(path="division_list.html")]
struct DivisionListTemplate {
  league: League,
  divisions: Vec<Division>,
}

#[derive(Template)]
#[template(path="league_list.html")]
struct LeagueListTemplate {
  leagues: Vec<League>,
}

#[derive(Template)]
#[template(path="game_list.html")]
struct GameListTemplate {
  division: Division,
  games: Vec<Game>,
}

#[derive(Template)]
#[template(path="partials/play_by_play_table.html")]
struct ShotsTableTemplate {
  shots: Vec<ShotDetails>,
}

#[derive(Template)]
#[template(path="game_score_page.html")]
struct GameScorePageTemplate {
  game: Game,
  division: Division,
  box_score: BoxScoreTemplate,
  team_stats: TeamGameStatsTemplate,
  individual_stats: IndividualGamePointsTableTemplate,
  play_by_play: ShotsTableTemplate,
}

#[derive(Template)]
#[template(path="player_page.html")]
pub struct PlayerPageTemplate {
  player: Player,
  league: League,
  league_stats: PlayerStats,
  lifetime_stats: PlayerStats,
}

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
    .route("/shots/", get(shots_all))
    .route("/test/", get(test_template))
    .route("/league/:id/", get(divisions_for_league_html))
    .route("/division/:id/", get(games_for_division_html))
    .route("/game/:id/", get(score_for_game_html))
    .route("/player/:name/", get(player_from_name))
    .with_state(state);
  let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
  println!("Listening on {}", addr);
  axum::Server::bind(&addr)
    .serve(router.into_make_service())
    .await
    .unwrap();
}

async fn player_from_name(State(server_config): State<ServerState>, Path(name): Path<String>) -> impl IntoResponse {
  let player = Player::from_name_case_insensitive(&server_config.db_pool, name)
    .await
    .unwrap();
  let latest_league = get_latest_league_for_player(&server_config.db_pool, &player)
    .await
    .unwrap()
    .unwrap();
  let latest_league_stats = get_league_player_stats(&server_config.db_pool, &player, &latest_league)
    .await
    .unwrap();
  let lifetime_stats = get_all_player_stats(&server_config.db_pool, &player)
    .await
    .unwrap();
  let html = PlayerPageTemplate {
    player,
    league: latest_league,
    league_stats: latest_league_stats,
    lifetime_stats,
  };
  (StatusCode::OK, html)
}

async fn test_template<'a>() -> HelloTemplate<'a> {
  HelloTemplate { name: "Tait", years: 24 }
}

macro_rules! get_all {
  ($crud_struct:ident, $func_name:ident) => {
    #[debug_handler]
    async fn $func_name(State(server_config): State<ServerState>) -> impl IntoResponse {
      let cruder = $crud_struct::all(&*server_config.db_pool)
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
      let cruder = $crud_struct::get(&*server_config.db_pool, id)
        .await
        .unwrap();
      (StatusCode::OK, Json(cruder))
    }
  }
}

async fn league_html(State(server_config): State<ServerState>) -> impl IntoResponse {
  let leagues = League::all(&*server_config.db_pool)
    .await
    .unwrap();
  let leagues_template = LeagueListTemplate {
    leagues
  };
  (StatusCode::OK, leagues_template)
}

async fn divisions_for_league_html(State(server_config): State<ServerState>, Path(league_id): Path<i32>) -> impl IntoResponse {
  let league = League::get(&*server_config.db_pool, league_id)
    .await
    .unwrap();
  let divisions = Division::by_league(&*server_config.db_pool, league_id)
    .await
    .unwrap();
  let html = DivisionListTemplate {
    league,
    divisions
  };
  (StatusCode::OK, html)
}

async fn games_for_division_html(State(server_config): State<ServerState>, Path(division_id): Path<i32>) -> impl IntoResponse {
  let division = Division::get(&*server_config.db_pool, division_id)
    .await
    .unwrap();
  let games = Game::by_division(&*server_config.db_pool, division.id)
    .await
    .unwrap();
  let games_template = GameListTemplate {
    division,
    games
  };
  (StatusCode::OK, games_template)
}
async fn score_for_game_html(State(server_config): State<ServerState>, Path(game_id): Path<i32>) -> impl IntoResponse {
  let game = sqlx::query_as::<_, Game>(
    "SELECT * FROM games WHERE id = $1;"
  )
  .bind(game_id)
  .fetch_one(&*server_config.db_pool)
  .await
  .unwrap();
  let division = Division::get(&*server_config.db_pool, game.division)
    .await
    .unwrap();
  let pbp = get_play_by_play_from_game(&server_config.db_pool, &game).await.unwrap();
  let score = get_score_from_game(&server_config.db_pool, &game).await.unwrap();
  let score_html = TeamGameStatsTemplate { teams: score };
  let goal_details = get_box_score_from_game(&server_config.db_pool, &game).await.unwrap();
  let goal_details_html = IndividualGamePointsTableTemplate { players: goal_details };
  let box_score = get_goals_from_game(&server_config.db_pool, &game).await.unwrap();
  let box_score_html = BoxScoreTemplate { goals: box_score };
  let pbp_html = ShotsTableTemplate {
    shots: pbp
  };
  let game_template = GameScorePageTemplate {
    division,
    game,
    box_score: box_score_html,
    team_stats: score_html,
    individual_stats: goal_details_html,
    play_by_play: pbp_html,
  };
  (StatusCode::OK, game_template)
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
*/

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
