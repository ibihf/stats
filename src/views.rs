use sqlx::FromRow;
use sqlx::PgPool;
use crate::model::Game;
use serde::{Serialize, Deserialize};

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct Score {
  pub home: i64,
  pub home_name: String,
  pub away: i64,
  pub away_name: String,
}

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct Notification {
  pub scorer_name: String,
  pub scorer_number: i32,
  pub position: String,
  pub scorer_team_name: String,
  pub period_name: String,
  pub period_time_left: i32,
}

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct PlayerStats {
  pub player_name: String,
  pub goals: i64,
  pub assists: i64,
  pub points: i64,
}


pub async fn get_box_score_from_game(pool: &PgPool, game: &Game) -> Result<Vec<PlayerStats>, sqlx::Error> {
  let query = format!(r#"
SELECT
  (
    SELECT COUNT(id)
    FROM shots
    WHERE shooter=players.id
      AND goal=true
      AND game=$1
  ) AS goals,
  (
    SELECT COUNT(id)
    FROM shots
    WHERE assistant=players.id
      AND goal=true
      AND game=$1
  ) AS assists,
  (
    SELECT COUNT(id)
    FROM shots
    WHERE (assistant=players.id
       OR shooter=players.id)
       AND game=$1
  ) AS points,
  players.name AS player_name
FROM players
JOIN shots ON shots.shooter=players.id OR shots.assistant=players.id
WHERE shots.game = $1
GROUP BY players.id
ORDER BY
  points DESC,
  goals DESC,
  players.name;
"#);
  sqlx::query_as::<_, PlayerStats>(&query)
    .bind(game.id)
    .fetch_all(pool)
    .await
}

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct DetailGoals {
  pub player_id: i32,
  pub player_name: String,
  pub number: i32,
  pub team_name: String,
  pub team_id: i32,
  pub time_remaining: i32,
  pub period_short_name: String,
  pub first_assist_name: Option<String>,
  pub first_assist_number: Option<i32>,
  pub first_assist_id: Option<i32>,
  pub second_assist_name: Option<String>,
  pub second_assist_id: Option<i32>,
  pub second_assist_number: Option<i32>,
}

pub async fn get_goals_from_game(pool: &PgPool, game: &Game) -> Result<Vec<NiceGoals>, sqlx::Error> {
}

pub async fn get_score_from_game(pool: &PgPool, game: &Game) -> Result<Score, sqlx::Error> {
  let query = format!(r#"
SELECT
  (
    SELECT COUNT(id)
    FROM shots
    WHERE game=$1
      AND goal=true
      AND shooter_team=$2
  ) AS home,
  (
    SELECT COUNT(id)
    FROM shots
    WHERE game=$1
      AND goal=true
      AND shooter_team=$3
  ) AS away,
  (
    SELECT name
    FROM teams
    WHERE id=$2
  ) AS home_name,
  (
    SELECT name
    FROM teams
    WHERE id=$3
  ) AS away_name
FROM games;
"#);
  sqlx::query_as::<_, Score>(&query)
    .bind(game.id)
    .bind(game.team_home)
    .bind(game.team_away)
    .fetch_one(pool)
    .await
}

async fn get_player_stats_overview(pool: PgPool) -> Result<Vec<PlayerStats>, sqlx::Error> {
  let query = r#"
SELECT
  (
    SELECT COUNT(id)
    FROM shots
    WHERE shooter=players.id
      AND goal=true
  ) AS goals,
  (
    SELECT COUNT(id)
    FROM shots
    WHERE assistant=players.id
      AND goal=true
  ) AS assists,
  (
    SELECT COUNT(id)
    FROM shots
    WHERE assistant=players.id
       OR shooter=players.id
  ) AS points,
  players.name AS player_name
FROM players
ORDER BY
  points DESC,
  goals DESC,
  players.name;
"#;
  let result = sqlx::query_as::<_, PlayerStats>(query)
    .fetch_all(&pool)
    .await;
  result
}

#[cfg(test)]
mod tests {
  use std::env;
  use crate::model::{
    Game,
  };
  use crate::views::{
    Notification,
    get_player_stats_overview,
    get_score_from_game,
    get_box_score_from_game,
  };

  #[test]
  fn check_box_score_from_game() {
    tokio_test::block_on(async move{
      let pool = db_connect().await;
      let game = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id=1;")
        .fetch_one(&pool)
        .await
        .unwrap();
      let scores = get_box_score_from_game(&pool, &game)
        .await
        .unwrap();
      println!("{scores:?}");
      assert_eq!(scores.get(0).unwrap().player_name, "Brian MacLean");
    })
  }
  
  #[test]
  fn check_game_score() {
    tokio_test::block_on(async move{
      let pool = db_connect().await;
      let game = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id=1;")
        .fetch_one(&pool)
        .await
        .unwrap();
      let score = get_score_from_game(&pool, &game)
        .await
        .unwrap();
      assert_eq!(score.away, 1);
      assert_eq!(score.home, 1);
    })
  }

  #[test]
  fn check_player_overall_stats() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let players_stats = get_player_stats_overview(pool).await.unwrap();
      for player_stats in players_stats {
        println!("{player_stats:?}");
      }
    })
  }

  #[test]
  fn check_notification_query() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let query = r#"
SELECT
  teams.name AS scorer_team_name,
  players.name AS scorer_name,
  positions.name AS position,
  team_players.player_number AS scorer_number,
  shots.period_time AS period_time_left,
  periods.name AS period_name
FROM
  shots
JOIN teams ON teams.id=shots.shooter_team
JOIN players ON players.id=shots.shooter
JOIN team_players ON team_players.player=players.id AND team_players.team=teams.id
JOIN periods ON periods.id=shots.period
JOIN positions ON positions.id=team_players.position;
"#;
      let result = sqlx::query_as::<_, Notification>(query)
        .fetch_one(&pool)
        .await
        .unwrap();
      let minutes = result.period_time_left / 60;
      let seconds = result.period_time_left % 60;
      println!("{0} {1} player #{3} {2} has scored! Time of the goal: {4}:{5} in the {6}",
        result.scorer_team_name,
        result.position,
        result.scorer_name,
        result.scorer_number,
        minutes,
        seconds,
        result.period_name
      );
    });
  }

  /// A simple function to connect to the database.
  async fn db_connect() -> sqlx::PgPool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set to run tests.");
    sqlx::postgres::PgPoolOptions::new()
      .max_connections(1)
      .connect(&db_url)
      .await
      .expect("Active database connection must be made")
  }
}
