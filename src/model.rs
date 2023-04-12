use sqlx::FromRow;
use sqlx::types::chrono::{DateTime, Utc};
use chrono::serde::ts_seconds;
use serde::{Serialize, Deserialize};

pub trait TableName {
  const TABLE_NAME: &'static str;
}
macro_rules! impl_table_name {
  ($ty:ident, $tname:literal) => {
    impl TableName for $ty {
      const TABLE_NAME: &'static str = $tname;
    }
  }
}

#[derive(FromRow, Serialize, Deserialize, Debug, ormx::Table)]
#[ormx(table = "supported_languages", id = id, insertable, deletable)]
pub struct Language {
  pub id: i32,
  pub native_name: String,
  pub short_name: String,
}

#[derive(FromRow, Serialize, Deserialize, Debug, ormx::Table)]
#[ormx(table = "leagues", id = id, insertable, deletable)]
pub struct League {
  #[ormx(default)]
  pub id: i32,
  pub name: String,
}
#[derive(FromRow, Serialize, Deserialize, Debug, ormx::Patch)]
#[ormx(table_name = "leagues", table = League, id = "id")]
pub struct NewLeague {
  pub name: String,
}

#[derive(FromRow, Serialize, Deserialize, Debug, ormx::Table)]
#[ormx(table = "divisions", id = id, insertable, deletable)]
pub struct Division {
  #[ormx(default)]
  pub id: i32,
  pub name: String,
  #[ormx(get_many(i32))]
  pub league: i32,
}

#[derive(FromRow, Serialize, Deserialize, Debug, ormx::Patch)]
#[ormx(table_name = "divisions", table = Division, id = "id")]
pub struct NewDivision {
  pub name: String,
  pub league: i32,
}

#[derive(FromRow, Serialize, Deserialize, Debug, ormx::Table)]
#[ormx(table = "teams", id = id, insertable, deletable)]
pub struct Team {
  #[ormx(default)]
  pub id: i32,
  pub name: String,
  pub division: i32,
  pub image: Option<String>,
}

#[derive(FromRow, Serialize, Deserialize, Debug, ormx::Patch)]
#[ormx(table_name = "teams", table = Team, id = "id")]
pub struct NewTeam {
  pub name: String,
  pub division: i32,
}

#[derive(FromRow, Serialize, Deserialize, Debug, ormx::Table)]
#[ormx(table = "players", id = id, insertable, deletable)]
pub struct Player {
  #[ormx(default)]
  pub id: i32,
  pub name: String,
  pub weight_kg: Option<i32>,
  pub height_cm: Option<i32>,
}

impl Player {
  pub async fn from_name_case_insensitive(pool: &sqlx::PgPool, name: String) -> Option<Player> {
    sqlx::query_as::<_, Player>("SELECT * FROM players WHERE REPLACE(UPPER(name), ' ', '-') LIKE UPPER($1);")
      .bind(name)
      .fetch_optional(pool)
      .await
      .unwrap()
  }
}

#[derive(FromRow, Deserialize, Serialize, Debug, ormx::Patch)]
#[ormx(table_name = "players", table = Player, id = "id")]
pub struct NewPlayer {
  pub name: String,
  pub weight_kg: Option<i32>,
  pub height_cm: Option<i32>,
}

#[derive(FromRow, Deserialize, Serialize, Debug, ormx::Table)]
#[ormx(table = "shots", id = id, insertable, deletable)]
pub struct Shot {
  #[ormx(default)]
  pub id: i32,
  pub shooter: i32,
  pub goalie: i32,
  pub assistant: Option<i32>,
  pub period: i32,
  pub period_time: i32,
  pub video_timestamp: Option<i32>,
  pub blocker: Option<i32>,
  pub on_net: bool,
  pub assistant_second: Option<i32>,
  pub goal: bool,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
}

#[derive(FromRow, Deserialize, Serialize, Debug, ormx::Table)]
#[ormx(table = "game_players", id = id, insertable, deletable)]
pub struct GamePlayer {
  #[ormx(default)]
  pub id: i32,
  pub team: i32,
  pub player: i32,
  pub position: i32,
  pub game: i32,
}

#[derive(FromRow, Deserialize, Serialize, Debug, ormx::Table)]
#[ormx(table = "games", id = id, insertable, deletable)]
pub struct Game {
  #[ormx(default)]
  pub id: i32,
  #[ormx(get_many(i32))]
  pub division: i32,
  pub name: String,
  pub team_home: i32,
  pub team_away: i32,
}

#[derive(FromRow, Deserialize, Serialize, Debug, ormx::Table)]
#[ormx(table = "periods", id = id, insertable, deletable)]
pub struct Period {
  pub id: i32,
  pub period_type: i32,
  #[ormx(get_many(i32))]
  pub game: i32,
}

impl_table_name!(GamePlayer, "game_players");
impl_table_name!(Player, "players");
impl_table_name!(League, "leagues");
impl_table_name!(Division, "divisions");
impl_table_name!(Team, "teams");
impl_table_name!(Shot, "shots");
impl_table_name!(Game, "games");
impl_table_name!(Period, "periods");

#[cfg(test)]
mod tests {
  use std::env;
  use crate::model::{
    GamePlayer,
    Player,
    League,
    Division,
    Team,
    Shot,
    TableName,
    Game,
  };

  #[test]
  fn test_get_player_from_name() {
    tokio_test::block_on(async move {
      let pool = db_connect().await;
      let player = Player::from_name_case_insensitive(&pool, "tait-hoyem".to_string()).await;
      assert!(player.is_some());
      let player = player.unwrap();
      assert_eq!(player.name, "Tait Hoyem");
    })
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
  generate_select_test!(GamePlayer, selec_game_player);
  generate_select_test!(Player, select_player);
  generate_select_test!(League, select_league);
  generate_select_test!(Division, select_division);
  generate_select_test!(Team, select_team);
  generate_select_test!(Shot, select_shot);
  generate_select_test!(Game, select_game);
}
