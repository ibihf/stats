use sqlx::FromRow;
use sqlx::PgPool;
use crate::model::{
  Player,
  Game,
  League,
};
use serde::{Serialize, Deserialize};

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct TeamStats {
  pub name: String,
  pub goals: i64,
  pub shots: i64,
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
  pub name: String,
  pub goals: i64,
  pub assists: i64,
  pub points: i64,
}

pub async fn get_box_score_from_game(pool: &PgPool, game: &Game) -> Result<Vec<PlayerStats>, sqlx::Error> {
  let query = r#"
SELECT
  (
    SELECT COUNT(shots.id)
    FROM shots
    JOIN periods ON periods.id=shots.period
    WHERE shooter=players.id
      AND goal=true
      AND periods.game=$1
  ) AS goals,
  (
    SELECT COUNT(shots.id)
    FROM shots
    JOIN periods ON periods.id=shots.period
    WHERE (assistant=players.id
       OR assistant_second=players.id)
       AND goal=true
       AND periods.game=$1
  ) AS assists,
  (
    SELECT COUNT(shots.id)
    FROM shots
    JOIN periods ON periods.id=shots.period
    WHERE (assistant=players.id
       OR assistant_second=players.id
       OR shooter=players.id)
       AND goal=true
       AND periods.game=$1
  ) AS points,
  players.name AS name
FROM players
JOIN shots ON shots.shooter=players.id OR shots.assistant=players.id
JOIN periods ON periods.id=shots.period
WHERE periods.game = $1
GROUP BY players.id
-- exclude players who do not have any points
-- NOTE: we can NOT use the aliased column "points" here, so we need to recalculate it.
-- This should not be a performance problem because the optimizer should deal with duplicate sub-queries.
HAVING
  (
    SELECT COUNT(shots.id)
    FROM shots
    JOIN periods ON periods.id=shots.period
    WHERE (assistant=players.id
       OR assistant_second=players.id
       OR shooter=players.id)
       AND goal=true
       AND periods.game=$1
  ) > 0
ORDER BY
  points DESC,
  goals DESC,
  players.name;
"#;
  sqlx::query_as::<_, PlayerStats>(query)
    .bind(game.id)
    .fetch_all(pool)
    .await
}

pub async fn get_latest_league_for_player(pool: &PgPool, player: &Player) -> Result<Option<League>, sqlx::Error> {
  let query =
r#"
SELECT leagues.*
FROM players
JOIN team_players ON team_players.player=players.id
JOIN teams ON teams.id=team_players.team
JOIN divisions ON divisions.id=teams.division
JOIN leagues ON leagues.id=divisions.league
WHERE players.id=$1
ORDER BY leagues.end_date DESC
LIMIT 1;
"#;
  sqlx::query_as::<_, League>(query)
    .bind(player.id)
    .fetch_optional(pool)
    .await
}

pub async fn get_league_player_stats(pool: &PgPool, player: &Player, league: &League) -> Result<PlayerStats, sqlx::Error> {
  let query = r#"
SELECT
  (
    SELECT COUNT(shots.id)
    FROM shots
    JOIN periods ON periods.id=shots.period
    JOIN games ON games.id=periods.game
    JOIN divisions ON divisions.id=games.division
    JOIN leagues ON leagues.id=divisions.league
    WHERE shots.goal=true
      AND shots.shooter=players.id
      AND leagues.id=$2
  ) AS goals,
  (
    SELECT COUNT(shots.id)
    FROM shots
    JOIN periods ON periods.id=shots.period
    JOIN games ON games.id=periods.game
    JOIN divisions ON divisions.id=games.division
    JOIN leagues ON leagues.id=divisions.league
    WHERE shots.goal=true
      AND leagues.id=$2
      AND (shots.assistant=players.id
       OR shots.assistant_second=players.id)
  ) AS assists,
  (
    SELECT COUNT(shots.id)
    FROM shots
    JOIN periods ON periods.id=shots.period
    JOIN games ON games.id=periods.game
    JOIN divisions ON divisions.id=games.division
    JOIN leagues ON leagues.id=divisions.league
    WHERE shots.goal=true
      AND leagues.id=$2
      AND (shots.shooter=players.id
       OR shots.assistant=players.id
       OR shots.assistant_second=players.id)
  ) AS points,
  players.name AS name
FROM players
WHERE id=$1;
"#;
  sqlx::query_as::<_, PlayerStats>(query)
    .bind(player.id)
    .bind(league.id)
    .fetch_one(pool)
    .await
}

pub async fn get_latest_stats(pool: &PgPool, player: &Player) -> Result<Vec<GoalDetails>, sqlx::Error> {
  let query = r#"
SELECT 
  shots.shooter AS player_id,
  shots.assistant AS first_assist_id,
  shots.assistant_second AS second_assist_id,
  (
    SELECT name
    FROM players
    WHERE id=shots.shooter
  ) AS player_name,
  (
    SELECT name
    FROM players
    WHERE id=shots.assistant
  ) AS first_assist_name,
  (
    SELECT name
    FROM players
    WHERE id=shots.assistant_second
  ) AS second_assist_name,
  (
    SELECT player_number
    FROM team_players
    WHERE player=shots.shooter
      AND team=shots.shooter_team
  ) AS player_number,
  (
    SELECT player_number
    FROM team_players
    WHERE player=shots.assistant
      AND team=shots.shooter_team
  ) AS first_assist_number,
  (
    SELECT player_number
    FROM team_players
    WHERE player=shots.assistant_second
      AND team=shots.shooter_team
  ) AS second_assist_number,
  teams.name AS team_name,
  teams.id AS team_id,
  shots.shooter_team AS player_team,
  shots.period_time AS time_remaining,
  period_types.id AS period_id,
  period_types.short_name AS period_short_name
FROM shots
JOIN periods ON shots.period=periods.id
JOIN period_types ON periods.period_type=period_types.id
JOIN teams ON shots.shooter_team=teams.id
WHERE shots.shooter=$1
ORDER BY
  shots.created_at DESC,
  periods.period_type DESC,
  shots.period_time ASC
LIMIT 5;
"#;
  sqlx::query_as::<_, GoalDetails>(query)
    .bind(player.id)
    .fetch_all(pool)
    .await
}

pub async fn get_all_player_stats(pool: &PgPool, player: &Player) -> Result<PlayerStats, sqlx::Error> {
  let query =r#"
SELECT
  (
    SELECT COUNT(id)
    FROM shots
    WHERE shots.goal=true
      AND shots.shooter=players.id
  ) AS goals,
  (
    SELECT COUNT(id)
    FROM shots
    WHERE shots.goal=true
      AND (shots.assistant=players.id
       OR shots.assistant_second=players.id)
  ) AS assists,
  (
    SELECT COUNT(id)
    FROM shots
    WHERE shots.goal=true
      AND (shots.shooter=players.id
       OR shots.assistant=players.id
       OR shots.assistant_second=players.id)
  ) AS points,
  players.name AS name
FROM players
WHERE id=$1;
"#;
  sqlx::query_as::<_, PlayerStats>(query)
    .bind(player.id)
    .fetch_one(pool)
    .await
}

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct GoalDetails {
  pub player_id: i32,
  pub player_name: String,
  pub player_number: i32,
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

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct ShotDetails {
  pub player_id: i32,
  pub player_name: String,
  pub player_number: i32,
  pub team_name: String,
  pub team_id: i32,
  pub is_goal: bool,
  pub time_remaining: i32,
  pub period_short_name: String,
  pub first_assist_name: Option<String>,
  pub first_assist_number: Option<i32>,
  pub first_assist_id: Option<i32>,
  pub second_assist_name: Option<String>,
  pub second_assist_id: Option<i32>,
  pub second_assist_number: Option<i32>,
}

pub async fn get_goals_from_game(pool: &PgPool, game: &Game) -> Result<Vec<GoalDetails>, sqlx::Error> {
  sqlx::query_as::<_, GoalDetails>(
r#"
SELECT 
  shots.shooter AS player_id,
  shots.assistant AS first_assist_id,
  shots.assistant_second AS second_assist_id,
  (
    SELECT name
    FROM players
    WHERE id=shots.shooter
  ) AS player_name,
  (
    SELECT name
    FROM players
    WHERE id=shots.assistant
  ) AS first_assist_name,
  (
    SELECT name
    FROM players
    WHERE id=shots.assistant_second
  ) AS second_assist_name,
  (
    SELECT player_number
    FROM team_players
    WHERE player=shots.shooter
      AND team=shots.shooter_team
  ) AS player_number,
  (
    SELECT player_number
    FROM team_players
    WHERE player=shots.assistant
      AND team=shots.shooter_team
  ) AS first_assist_number,
  (
    SELECT player_number
    FROM team_players
    WHERE player=shots.assistant_second
      AND team=shots.shooter_team
  ) AS second_assist_number,
  teams.name AS team_name,
  teams.id AS team_id,
  shots.shooter_team AS player_team,
  shots.period_time AS time_remaining,
  period_types.id AS period_id,
  period_types.short_name AS period_short_name
FROM shots
JOIN periods ON shots.period=periods.id
JOIN period_types ON periods.period_type=period_types.id
JOIN teams ON shots.shooter_team=teams.id
WHERE shots.goal=true
  AND periods.game=$1
ORDER BY
  periods.period_type ASC,
  shots.period_time DESC;
"#)
    .bind(game.id)
    .fetch_all(pool)
    .await
}

pub async fn get_play_by_play_from_game(pool: &PgPool, game: &Game) -> Result<Vec<ShotDetails>, sqlx::Error> {
  sqlx::query_as::<_, ShotDetails>(
r#"
SELECT 
  shots.shooter AS player_id,
  shots.assistant AS first_assist_id,
  shots.assistant_second AS second_assist_id,
  shots.goal AS is_goal,
  (
    SELECT name
    FROM players
    WHERE id=shots.shooter
  ) AS player_name,
  (
    SELECT name
    FROM players
    WHERE id=shots.assistant
  ) AS first_assist_name,
  (
    SELECT name
    FROM players
    WHERE id=shots.assistant_second
  ) AS second_assist_name,
  (
    SELECT player_number
    FROM team_players
    WHERE player=shots.shooter
      AND team=shots.shooter_team
  ) AS player_number,
  (
    SELECT player_number
    FROM team_players
    WHERE player=shots.assistant
      AND team=shots.shooter_team
  ) AS first_assist_number,
  (
    SELECT player_number
    FROM team_players
    WHERE player=shots.assistant_second
      AND team=shots.shooter_team
  ) AS second_assist_number,
  teams.name AS team_name,
  teams.id AS team_id,
  shots.shooter_team AS player_team,
  shots.period_time AS time_remaining,
  period_types.id AS period_id,
  period_types.short_name AS period_short_name
FROM shots
JOIN periods ON shots.period=periods.id
JOIN period_types ON periods.period_type=period_types.id
JOIN teams ON shots.shooter_team=teams.id
WHERE periods.game=$1
ORDER BY
  periods.period_type ASC,
  shots.period_time DESC;
"#)
    .bind(game.id)
    .fetch_all(pool)
    .await
}

pub async fn get_score_from_game(pool: &PgPool, game: &Game) -> Result<Vec<TeamStats>, sqlx::Error> {
  let query = r#"
SELECT 
  (
    SELECT COUNT(shots.id)
    FROM shots
    JOIN periods ON periods.id=shots.period
    WHERE periods.game=$1
      AND shots.goal=true
      AND shots.shooter_team=teams.id
  ) AS goals,
  (
    SELECT COUNT(shots.id)
    FROM shots
    JOIN periods ON periods.id=shots.period
    WHERE periods.game=$1
      AND shooter_team=teams.id
  ) AS shots,
  teams.name AS name
FROM games
JOIN teams ON teams.id=games.team_home OR teams.id=games.team_away
WHERE games.id=$1;
"#;
  sqlx::query_as::<_, TeamStats>(query)
    .bind(game.id)
    .fetch_all(pool)
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
    WHERE (assistant=players.id OR assistant_second=players.id)
      AND goal=true
  ) AS assists,
  (
    SELECT COUNT(id)
    FROM shots
    WHERE assistant=players.id
       OR shooter=players.id
  ) AS points,
  players.name AS name
FROM players
ORDER BY
  points DESC,
  goals DESC,
  players.name;
"#;
  sqlx::query_as::<_, PlayerStats>(query)
    .fetch_all(&pool)
    .await
}

#[cfg(test)]
mod tests {
  use std::env;
  use crate::model::{
    Game,
    Player,
    League,
  };
  use crate::views::{
    Notification,
    get_player_stats_overview,
    get_score_from_game,
    get_goals_from_game,
    get_box_score_from_game,
    get_latest_league_for_player,
    get_league_player_stats,
    get_all_player_stats,
    get_latest_stats,
  };

  #[test]
  fn get_latest_stats_of_player() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let player = sqlx::query_as::<_, Player>("SELECT * FROM id=2;")
        .fetch_one(&pool)
        .await
        .unwrap();
      let latest = get_latest_stats(&pool, &player)
        .await
        .unwrap();
    })
  }

  #[test]
  fn check_all_player_stats() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let player = sqlx::query_as::<_, Player>("SELECT * FROM players WHERE id=2;")
        .fetch_one(&pool)
        .await
        .unwrap();
      let stats = get_all_player_stats(&pool, &player).await.unwrap();
      assert_eq!(stats.name, "Hillary Scanlon");
    })
  }

  #[test]
  fn check_league_player_stats() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let league = sqlx::query_as::<_, League>("SELECT * FROM leagues WHERE id=1;")
        .fetch_one(&pool)
        .await
        .unwrap();
      let player = sqlx::query_as::<_, Player>("SELECT * FROM players WHERE id=2;")
        .fetch_one(&pool)
        .await
        .unwrap();
      let stats = get_league_player_stats(&pool, &player, &league).await.unwrap();
      assert_eq!(stats.name, "Hillary Scanlon");
    })
  }

  #[test]
  fn check_latest_league_for_player() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let player = sqlx::query_as::<_, Player>("SELECT * FROM players WHERE id=5")
        .fetch_one(&pool)
        .await
        .unwrap();
      let league = get_latest_league_for_player(&pool, &player)
        .await
        .unwrap()
        .unwrap();
      assert_eq!(league.id, 1);
    })
  }

  #[test]
  fn check_score_details_from_game() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let game = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id=1;")
        .fetch_one(&pool)
        .await
        .unwrap();
      let scores = get_goals_from_game(&pool, &game)
        .await
        .unwrap();
      println!("{scores:?}");
    })
  }

  #[test]
  fn check_box_score_from_game() {
    tokio_test::block_on(async move{
      let pool = db_connect().await;
      let game = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id=4;")
        .fetch_one(&pool)
        .await
        .unwrap();
      let scores = get_box_score_from_game(&pool, &game)
        .await
        .unwrap();
      println!("{scores:?}");
      let second_top_scorer = scores.get(1).unwrap();
      assert_eq!(second_top_scorer.name, "Allyssa Foulds");
      assert_eq!(second_top_scorer.goals, 1, "Allysa should have 1 goal..");
      assert_eq!(second_top_scorer.assists, 2, "Allyssa should have 2 assists.");
      assert_eq!(second_top_scorer.points, 3, "Allysa should have 3 points.");
      assert_eq!(scores.len(), 8, "Players which did not receive any points should not be in the box score.");
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
      assert_eq!(score.get(0).unwrap().goals, 1);
      assert_eq!(score.get(1).unwrap().goals, 1);
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
  period_types.name AS period_name
FROM
  shots
JOIN teams ON teams.id=shots.shooter_team
JOIN players ON players.id=shots.shooter
JOIN team_players ON team_players.player=players.id AND team_players.team=teams.id
JOIN periods ON periods.id=shots.period
JOIN period_types ON period_types.id=periods.period_type
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
