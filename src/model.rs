use sqlx::FromRow;
use sqlx::types::chrono::{DateTime, Utc};
use chrono::serde::{ts_seconds, ts_seconds_option};
use serde::{Serialize, Deserialize};

pub trait TableName {
  const TABLE_NAME: &'static str;
}
macro_rules! impl_table_name {
  ($ty:ident, $tname:expr) => {
    impl TableName for $ty {
      const TABLE_NAME: &'static str = $tname;
    }
  }
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct League {
  pub id: i32,
  pub name: String,
  #[serde(with = "ts_seconds")]
  pub start_date: DateTime<Utc>,
  #[serde(with = "ts_seconds_option")]
  pub end_date: Option<DateTime<Utc>>,
}
#[derive(FromRow, Serialize, Deserialize)]
pub struct NewLeague {
  pub name: String,
  #[serde(with = "ts_seconds")]
  pub start_date: DateTime<Utc>,
  #[serde(with = "ts_seconds_option")]
  pub end_date: Option<DateTime<Utc>>,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct Division {
  pub id: i32,
  pub name: String,
  pub league: i32,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct NewDivision {
  pub id: i32,
  pub name: String,
  pub league: i32,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct Team {
  pub id: i32,
  pub name: String,
  pub division: i32,
  pub image: Option<String>,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct NewTeam {
  pub name: String,
  pub division: i32,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct Player {
  pub id: i32,
  pub name: String,
  pub weight_kg: Option<i32>,
  pub height_cm: Option<i32>,
}

#[derive(FromRow, Deserialize, Serialize)]
pub struct NewPlayer {
  pub name: String,
  pub weight_kg: Option<i32>,
  pub height_cm: Option<i32>,
}

#[derive(FromRow, Deserialize, Serialize)]
pub struct Shot {
  pub id: i32,
  pub shooter_team: i32,
  pub goalie: i32,
  pub assistant: Option<i32>,
  pub game: i32,
  pub period: i32,
  pub period_time: i32,
  pub video_timestamp: Option<i32>,
}

#[derive(FromRow, Deserialize, Serialize)]
pub struct TeamPlayer {
  pub id: i32,
  pub team: i32,
  pub player: i32,
  pub position: i32,
}

#[derive(FromRow, Deserialize, Serialize)]
pub struct Game {
  pub id: i32,
  pub division: i32,
  pub name: String,
  pub team_home: i32,
  pub team_away: i32,
}

impl_table_name!(TeamPlayer, "team_players");
impl_table_name!(Player, "players");
impl_table_name!(League, "leagues");
impl_table_name!(Division, "divisions");
impl_table_name!(Team, "teams");
impl_table_name!(Shot, "shots");
impl_table_name!(Game, "games");

#[cfg(test)]
mod tests {
  use std::env;
  use crate::model::{
    TeamPlayer,
    Player,
    League,
    Division,
    Team,
    Shot,
    TableName,
    Game,
  };

  /// A simple function to connect to the database.
  async fn db_connect() -> sqlx::PgPool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set to run tests.");
    sqlx::postgres::PgPoolOptions::new()
      .max_connections(1)
      .connect(&db_url)
      .await
      .expect("Active database connection must be made")
  }

  /// This macro generates a test that will `SELECT` all records for a table.
  /// Then, it checks that
  /// 1. The table rows gets deserialized correctly.
  /// 2. There is at least one row.
  macro_rules! generate_select_test {
    ($ret_type:ident, $func_name:ident) => {
      #[test]
      fn $func_name() {
        tokio_test::block_on(async move {
          let pool = db_connect().await;
          let results = sqlx::query_as::<_, $ret_type>(
            &format!(
              "SELECT * FROM {};",
              <$ret_type as TableName>::TABLE_NAME
            )
          )
          .fetch_all(&pool)
          .await
          .unwrap();
          // check that there is at least one result item
          assert!(results.len() > 0, "There must be at least one result in the table.");
        });
      }
    }
  }
  generate_select_test!(TeamPlayer, select_team_player);
  generate_select_test!(Player, select_player);
  generate_select_test!(League, select_league);
  generate_select_test!(Division, select_division);
  generate_select_test!(Team, select_team);
  generate_select_test!(Shot, select_shot);
  generate_select_test!(Game, select_game);
}
