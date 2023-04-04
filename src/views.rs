use sqlx::FromRow;
use sqlx::PgPool;
use crate::model::{
  Player,
  Game,
  League,
  Division,
  Period,
};
use serde::{Serialize, Deserialize};

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct TeamStats {
  pub name: String,
  pub goals: i64,
  pub shots: i64,
}

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct IihfGameStats {
  pub team_name: String,
  pub team_id: i32,
  pub points: i64,
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

impl Game {
  pub async fn box_score(pool: &PgPool, id: i32) -> Result<Vec<PlayerStats>, sqlx::Error> {
    let query = r#"
  SELECT
    COUNT(shots.id) AS points,
    COUNT(CASE WHEN shots.shooter = game_players.id THEN shots.id END) AS goals,
    COUNT(CASE WHEN shots.assistant = game_players.id OR shots.assistant_second = game_players.id THEN shots.id END) AS assists,
    players.name
  FROM game_players
  JOIN players ON game_players.player = players.id
  LEFT JOIN shots
    ON shots.goal=true
   AND (shots.shooter=game_players.id
    OR shots.assistant=game_players.id
    OR shots.assistant_second=game_players.id)
  WHERE game_players.game=$1
  GROUP BY
    game_players.id,
    players.name
  HAVING COUNT(shots.id) > 0
  ORDER BY
    points DESC,
    goals DESC;
  "#;
    sqlx::query_as::<_, PlayerStats>(query)
      .bind(id)
      .fetch_all(pool)
      .await
  }
  pub async fn goals(pool: &PgPool, id: i32) -> Result<Vec<GoalDetails>, sqlx::Error> {
    sqlx::query_as::<_, GoalDetails>(
  r#"
  SELECT 
    shots.shooter AS player_id,
    shots.assistant AS first_assist_id,
    shots.assistant_second AS second_assist_id,
    players.name AS player_name,
    p_assist.name AS first_assist_name,
    p_assist_second.name AS second_assist_name,
    game_players.player_number AS player_number,
    gp_assist.player_number AS first_assist_number,
    gp_assist_second.player_number AS second_assist_number,
    teams.name AS team_name,
    teams.id AS team_id,
    shots.period_time AS time_remaining,
    period_types.id AS period_id,
    period_types.short_name AS period_short_name
  FROM shots
  JOIN game_players ON game_players.id=shots.shooter
  JOIN players ON players.id=game_players.player
  LEFT JOIN game_players gp_assist ON gp_assist.id=shots.assistant
  LEFT JOIN players p_assist ON p_assist.id=gp_assist.player
  LEFT JOIN game_players gp_assist_second ON gp_assist.id=shots.assistant_second
  LEFT JOIN players p_assist_second ON p_assist.id=gp_assist_second.player
  JOIN teams ON teams.id=game_players.team
  JOIN periods ON periods.id=shots.period
  JOIN period_types ON period_types.id=periods.period_type
  JOIN games ON games.id=periods.game
  WHERE shots.goal=true
    AND games.id=$1
  ORDER BY
    periods.period_type ASC,
    shots.period_time DESC;
  "#)
      .bind(id)
      .fetch_all(pool)
      .await
  }
  /// Returns the number of points using IIHF scoring rules for each team.
  pub async fn iihf_score(pool: &PgPool, game_id: i32) -> Result<Vec<IihfGameStats>, sqlx::Error> {
    let query = r#"
  SELECT 
    COUNT(CASE WHEN shots.goal = true THEN shots.id END) AS points,
    teams.id AS team_id,
    teams.name AS team_name
  FROM games
  JOIN periods ON periods.game=games.id
  JOIN shots ON shots.period=periods.id
  JOIN game_players ON game_players.id=shots.shooter
  JOIN teams ON teams.id=game_players.team
  WHERE games.id=$1
  GROUP BY teams.id;
  "#;
    sqlx::query_as::<_, IihfGameStats>(query)
      .bind(game_id)
      .fetch_all(pool)
      .await
  }
  /// Returns the number of shots and goals for each team in the game.
  pub async fn score(pool: &PgPool, game_id: i32) -> Result<Vec<TeamStats>, sqlx::Error> {
    let query = r#"
  SELECT 
    COUNT(CASE WHEN shots.goal = true THEN shots.id END) AS goals,
    COUNT(shots.id) AS shots,
    teams.name AS name
  FROM games
  JOIN periods ON periods.game=games.id
  JOIN shots ON shots.period=periods.id
  JOIN game_players ON game_players.id=shots.shooter
  JOIN teams ON teams.id=game_players.team
  WHERE games.id=$1
  GROUP BY teams.id;
  "#;
    sqlx::query_as::<_, TeamStats>(query)
      .bind(game_id)
      .fetch_all(pool)
      .await
  }
}

impl Player {
  pub async fn latest_league(pool: &PgPool, id: i32) -> Result<Option<League>, sqlx::Error> {
    let query =
  r#"
  SELECT leagues.*
  FROM players
  JOIN game_players ON game_players.player=players.id
  JOIN games ON games.id=game_players.game
  JOIN teams ON teams.id=game_players.team
  JOIN divisions ON divisions.id=teams.division
  JOIN leagues ON leagues.id=divisions.league
  WHERE players.id=$1
  ORDER BY games.end_at DESC
  LIMIT 1;
  "#;
    sqlx::query_as::<_, League>(query)
      .bind(id)
      .fetch_optional(pool)
      .await
  }
  pub async fn latest_stats(pool: &PgPool, id: i32) -> Result<Vec<GoalDetails>, sqlx::Error> {
    let query = 
  r#"
  SELECT 
    players.id AS player_id,
    p_assist.id AS first_assist_id,
    p_assist_second.id AS second_assist_id,
    players.name AS player_name,
    p_assist.name AS first_assist_name,
    p_assist_second.name AS second_assist_name,
    game_players.player_number AS player_number,
    gp_assist.player_number AS first_assist_number,
    gp_assist_second.player_number AS second_assist_number,
    teams.name AS team_name,
    teams.id AS team_id,
    shots.period_time AS time_remaining,
    period_types.id AS period_id,
    period_types.short_name AS period_short_name
  FROM shots
  JOIN game_players ON game_players.id=shots.shooter
  JOIN players ON players.id=game_players.player
  JOIN teams ON teams.id=game_players.team
  LEFT JOIN game_players gp_assist ON gp_assist.id=shots.assistant
  LEFT JOIN players p_assist ON p_assist.id=gp_assist.player
  LEFT JOIN game_players gp_assist_second ON gp_assist_second.id=shots.assistant_second
  LEFT JOIN players p_assist_second ON p_assist_second.id=gp_assist_second.id
  JOIN periods ON shots.period=periods.id
  JOIN period_types ON period_types.id=periods.period_type
  WHERE players.id=$1
  ORDER BY
    shots.created_at DESC,
    periods.period_type DESC,
    shots.period_time ASC
  LIMIT 5;
  "#;
    sqlx::query_as::<_, GoalDetails>(&query)
      .bind(id)
      .fetch_all(pool)
      .await
  }
  pub async fn lifetime_stats(pool: &PgPool, id: i32) -> Result<PlayerStats, sqlx::Error> {
    let query =r#"
  SELECT
    COUNT(goals) AS goals,
    COUNT(assists) AS assists,
    COUNT(points) AS points,
    players.name AS name
  FROM players
  JOIN game_players ON game_players.player=players.id
  LEFT JOIN shots points
    ON (points.shooter=game_players.id
    OR points.assistant=game_players.id
    OR points.assistant_second=game_players.id)
   AND points.goal=true
  LEFT JOIN shots goals
    ON goals.shooter=game_players.id
   AND goals.goal=true
  LEFT JOIN shots assists
    ON (points.assistant=game_players.id
    OR points.assistant_second=game_players.id)
   AND points.goal=true
  WHERE players.id=$1
  GROUP BY players.id;
  "#;
    sqlx::query_as::<_, PlayerStats>(query)
      .bind(id)
      .fetch_one(pool)
      .await
  }
}
async fn get_player_stats_overview(pool: PgPool) -> Result<Vec<PlayerStats>, sqlx::Error> {
  let query = r#"
SELECT
  COUNT(shots.id) AS points,
  COUNT(CASE WHEN shots.shooter = game_players.id THEN shots.id END) AS goals,
  COUNT(CASE WHEN shots.assistant = game_players.id OR shots.assistant_second = game_players.id THEN shots.id END) AS assists,
  players.name
FROM game_players
JOIN players ON game_players.player = players.id
LEFT JOIN shots
  ON shots.goal=true
 AND (shots.shooter=game_players.id
  OR shots.assistant=game_players.id
  OR shots.assistant_second=game_players.id)
GROUP BY
  game_players.id,
  players.name
ORDER BY
  points DESC,
  goals DESC;
"#;
  sqlx::query_as::<_, PlayerStats>(query)
    .fetch_all(&pool)
    .await
}

impl League {
  pub async fn player_stats(pool: &PgPool, player_id: i32, league_id: i32) -> Result<PlayerStats, sqlx::Error> {
    let query = r#"
  SELECT
    COUNT(goals.id) AS goals,
    COUNT(assists.id) AS assists,
    COUNT(points.id) AS points,
    players.name AS name
  FROM players
  JOIN game_players ON game_players.player=players.id
  LEFT JOIN shots goals
    ON goals.goal=true
   AND goals.shooter=game_players.id
  LEFT JOIN shots assists
    ON assists.goal=true
   AND (assists.assistant=game_players.id
    OR assists.assistant_second=game_players.id) 
  LEFT JOIN shots points
    ON points.goal=true
   AND (points.shooter=game_players.id
    OR points.assistant=game_players.id
    OR points.assistant_second=game_players.id)
  JOIN games ON games.id=game_players.game
  JOIN divisions ON divisions.id=games.division
  JOIN leagues ON leagues.id=divisions.league
  WHERE leagues.id=$1
    AND players.id=$2
  GROUP BY players.id;
  "#;
    sqlx::query_as::<_, PlayerStats>(query)
      .bind(league_id)
      .bind(player_id)
      .fetch_one(pool)
      .await
  }
}



#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct GoalDetails {
  pub player_id: i32,
  pub player_name: String,
  pub player_number: i32,
  pub team_name: String,
  pub team_id: i32,
  pub time_remaining: i32,
  pub period_id: i32,
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


pub async fn get_play_by_play_from_game(pool: &PgPool, game: &Game) -> Result<Vec<ShotDetails>, sqlx::Error> {
  sqlx::query_as::<_, ShotDetails>(
r#"
SELECT 
  shots.shooter AS player_id,
  shots.assistant AS first_assist_id,
  shots.assistant_second AS second_assist_id,
  shots.goal AS is_goal,
  players.name AS player_name,
  p_assistant.name AS first_assist_name,
  p_assistant_second.name AS second_assist_name,
  game_players.player_number AS player_number,
  gp_assistant.player_number AS first_assist_number,
  gp_assistant_second.player_number AS second_assist_number,
  teams.name AS team_name,
  teams.id AS team_id,
  shots.period_time AS time_remaining,
  period_types.id AS period_id,
  period_types.short_name AS period_short_name
FROM shots
JOIN game_players ON game_players.id=shots.shooter
JOIN players ON players.id=game_players.player
JOIN teams ON teams.id=game_players.team
LEFT JOIN game_players gp_assistant ON gp_assistant.id=shots.assistant
LEFT JOIN players p_assistant ON p_assistant.id=gp_assistant.player
LEFT JOIN game_players gp_assistant_second ON gp_assistant_second.id=shots.assistant_second
LEFT JOIN players p_assistant_second ON p_assistant_second.id=gp_assistant_second.player
JOIN periods ON shots.period=periods.id
JOIN period_types ON periods.period_type=period_types.id
WHERE periods.game=$1
ORDER BY
  periods.period_type ASC,
  shots.period_time DESC;
"#)
    .bind(game.id)
    .fetch_all(pool)
    .await
}



#[cfg(test)]
mod tests {
  use std::env;
  use ormx::Table;
  use crate::model::{
    Game,
    Player,
    League,
  };
  use crate::views::{
    Notification,
    get_player_stats_overview,
    get_play_by_play_from_game,
  };
  
  #[test]
  fn check_play_by_play() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let game = Game::get(&pool, 3)
        .await
        .unwrap();
      let pbp = get_play_by_play_from_game(&pool, &game)
        .await
        .unwrap();
    })
  }

  #[test]
  fn get_latest_stats_of_player() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let player = Player::get(&pool, 2)
        .await
        .unwrap();
      let latest = Player::latest_stats(&pool, player.id)
        .await
        .unwrap();
    })
  }

  #[test]
  fn check_league_player_stats() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let league = League::get(&pool, 1)
        .await
        .unwrap();
      let player = Player::get(&pool, 2)
        .await
        .unwrap();
      let stats = League::player_stats(&pool, player.id, league.id).await.unwrap();
      assert_eq!(stats.name, "Hillary Scanlon");
    })
  }

  #[test]
  fn check_latest_league_for_player() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let player = Player::get(&pool, 5)
        .await
        .unwrap();
      let league = Player::latest_league(&pool, player.id)
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
      let game = Game::get(&pool, 3)
        .await
        .unwrap();
      let scores = Game::goals(&pool, game.id)
        .await
        .unwrap();
      println!("{scores:?}");
    })
  }

  #[test]
  fn check_box_score_from_game() {
    tokio_test::block_on(async move{
      let pool = db_connect().await;
      let game = Game::get(&pool, 4)
        .await
        .unwrap();
      let scores = Game::box_score(&pool, game.id)
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
  fn check_iihf_score() {
    tokio_test::block_on(async move{
      let pool = db_connect().await;
      let game = Game::get(&pool, 1)
        .await
        .unwrap();
      let score = Game::iihf_score(&pool, game.id)
        .await
        .unwrap();
      assert_eq!(score.get(0).unwrap().points, 3);
      assert_eq!(score.get(0).unwrap().team_name, "Bullseye");
    })
  }
  
  #[test]
  fn check_game_score() {
    tokio_test::block_on(async move{
      let pool = db_connect().await;
      let game = Game::get(&pool, 1)
        .await
        .unwrap();
      let score = Game::score(&pool, game.id)
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
  fn check_lifetime_stats() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let lifetime_stats = Player::lifetime_stats(&pool, 5)
        .await
        .unwrap();
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
  game_players.player_number AS scorer_number,
  shots.period_time AS period_time_left,
  period_types.name AS period_name
FROM
  shots
JOIN game_players ON game_players.id=shots.shooter
JOIN players ON players.id=game_players.player
JOIN teams ON teams.id=game_players.team
JOIN periods ON periods.id=shots.period
JOIN period_types ON period_types.id=periods.period_type
JOIN positions ON positions.id=game_players.position;
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
