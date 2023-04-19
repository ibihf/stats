use chrono::serde::ts_seconds;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::FromRow;

pub trait TableName {
    const TABLE_NAME: &'static str;
}
pub trait NameTableName {
    const NAME_TABLE_NAME: &'static str;
      const NAME_TABLE_FK_NAME: &'static str;
}
macro_rules! impl_table_name {
    ($ty:ident, $tname:literal) => {
        impl TableName for $ty {
            const TABLE_NAME: &'static str = $tname;
        }
    };
}
macro_rules! impl_name_table_name {
    ($ty:ident, $tname:literal, $fk_name:literal) => {
        impl NameTableName for $ty {
            const NAME_TABLE_NAME: &'static str = $tname;
            const NAME_TABLE_FK_NAME: &'static str = $fk_name;
        }
    };
}

#[derive(FromRow, Serialize, Deserialize, Debug, ormx::Table)]
#[ormx(table = "supported_languages", id = id, insertable, deletable)]
pub struct Language {
    #[ormx(default)]
    pub id: i32,
    pub native_name: String,
    pub short_name: String,
}

macro_rules! impl_localized_get {
  ($struct:ident, $func:ident) => {
    impl $struct {
      pub async fn get(pool: &sqlx::PgPool, id: i32, lang: crate::SupportedLanguage) -> Result<Option<Self>, sqlx::Error> {
       let query =  const_format::formatcp!(r#"
SELECT
  {0}.*,
  {1}({0}.id, $2) AS name
FROM {0}
WHERE {0}.id = $1;"#,
            stringify!(<$struct as TableName>::TABLE_NAME),
            stringify!($func),
        );
        sqlx::query_as::<_, $struct>(&query)
        .bind(id)
        .bind(lang)
        .fetch_optional(pool)
        .await
      }
    }
  };
}
macro_rules! impl_localized_get_by_many {
  ($struct:ident, $by:ident) => {
    impl $struct {
      #[rename::rename_fn(prepend="by_")]
      pub async fn $by(pool: &sqlx::PgPool, by: i32, lang: crate::SupportedLanguage) -> Result<Vec<Self>, sqlx::Error> {
        let query = format!(r#"
SELECT
  {0}.*,
  -- if available, use the localized name,
  -- if not, choose the default name (in English, CA)
  -- if not, see if there is any name attached (there should be at least some language selected)
  COALESCE(
    localized_name.name,
    default_name.name,
    any_name.name
  ) AS name
FROM {0}
-- search for the proper, localized name
LEFT JOIN {1} localized_name
  ON localized_name.{2} = {0}.id
 AND localized_name.language = $2
-- also look for the name in the default language (English, CA)
LEFT JOIN {1} default_name
  ON default_name.{2} = {0}.id
 AND default_name.language=1
-- also look for ANY matching name in ANY locale
LEFT JOIN {1} any_name
  ON any_name.{2} = {0}.id
WHERE {0}.{3} = $1;"#,
            <$struct as TableName>::TABLE_NAME,
            <$struct as NameTableName>::NAME_TABLE_NAME,
            <$struct as NameTableName>::NAME_TABLE_FK_NAME,
            stringify!($by),
        );
        sqlx::query_as::<_, $struct>(&query)
        .bind(by)
        .bind(lang)
        .fetch_all(pool)
        .await
      }
    }
  };
}
macro_rules! impl_localized_all {
  ($struct:ident) => {
    impl $struct {
      pub async fn all(pool: &sqlx::PgPool, lang: crate::SupportedLanguage) -> Result<Vec<Self>, sqlx::Error> {
        let query = format!(r#"
SELECT
  {0}.*,
  -- if available, use the localized name,
  -- if not, choose the default name (in English, CA)
  -- if not, see if there is any name attached (there should be at least some language selected)
  COALESCE(
    localized_name.name,
    default_name.name,
    any_name.name
  ) AS name
FROM {0}
-- search for the proper, localized name
LEFT JOIN {1} localized_name
  ON localized_name.{2} = {0}.id
 AND localized_name.language = $1
-- also look for the name in the default language (English, CA)
LEFT JOIN {1} default_name
  ON default_name.{2} = {0}.id
 AND default_name.language=1
-- also look for ANY matching name in ANY locale
LEFT JOIN {1} any_name
  ON any_name.{2} = {0}.id;"#,
            <$struct as TableName>::TABLE_NAME,
            <$struct as NameTableName>::NAME_TABLE_NAME,
            <$struct as NameTableName>::NAME_TABLE_FK_NAME,
        );
        sqlx::query_as::<_, $struct>(&query)
        .bind(lang)
        .fetch_all(pool)
        .await
      }
    }
  };
}

#[derive(FromRow, Serialize, Deserialize, Debug, NameTableName)]
#[table_names(table_name = "leagues", name_func = "league_name", name_table_name = "league_names", name_table_name_fk = "league")]
pub struct League {
    //#[ormx(default)]
    pub id: i32,
    pub name: Option<String>,
}
//impl_localized_get!(League, league_name);
impl_localized_all!(League);
/*
#[derive(FromRow, Serialize, Deserialize, Debug, ormx::Patch)]
#[ormx(table_name = "leagues", table = League, id = "id")]
pub struct NewLeague {
  pub name: String,
}
*/

#[derive(FromRow, Serialize, Deserialize, Debug)]
//#[ormx(table = "divisions", id = id, insertable, deletable)]
pub struct Division {
    //#[ormx(default)]
    pub id: i32,
    //#[ormx(get_many(i32))]
    pub league: i32,
    pub name: Option<String>,
}
impl_localized_get!(Division, division_name);
impl_localized_get_by_many!(Division, league);
impl_localized_all!(Division);

#[derive(FromRow, Serialize, Deserialize, Debug)]
//#[ormx(table_name = "divisions", table = Division, id = "id")]
pub struct NewDivision {
    pub league: i32,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
//#[ormx(table = "teams", id = id, insertable, deletable)]
pub struct Team {
    //#[ormx(default)]
    pub id: i32,
    pub division: i32,
    pub image: Option<String>,
    pub name: Option<String>,
}
impl_localized_get!(Team, team_name);

/*
#[derive(FromRow, Serialize, Deserialize, Debug, ormx::Patch)]
#[ormx(table_name = "teams", table = Team, id = "id")]
pub struct NewTeam {
    pub name: String,
    pub division: i32,
}
*/

#[derive(FromRow, Serialize, Deserialize, Debug, ormx::Table)]
#[ormx(table = "players", id = id, insertable, deletable)]
pub struct Player {
    //#[ormx(default)]
    pub id: i32,
    pub first_names: String,
    pub last_name: String,
    pub weight_kg: Option<i32>,
    pub height_cm: Option<i32>,
}

impl Player {
    pub async fn from_name_case_insensitive(pool: &sqlx::PgPool, name: String) -> Option<Player> {
        sqlx::query_as::<_, Player>(
            "SELECT * FROM players WHERE REPLACE(UPPER(name), ' ', '-') LIKE UPPER($1);",
        )
        .bind(name)
        .fetch_optional(pool)
        .await
        .unwrap()
    }
}

#[derive(FromRow, Deserialize, Serialize, Debug, ormx::Patch)]
#[ormx(table_name = "players", table = Player, id = "id")]
pub struct NewPlayer {
    pub first_names: String,
    pub last_name: String,
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

#[derive(FromRow, Deserialize, Serialize, Debug)]
//#[ormx(table = "games", id = id, insertable, deletable)]
pub struct Game {
    //#[ormx(default)]
    pub id: i32,
    //#[ormx(get_many(i32))]
    pub division: i32,
    pub team_home: i32,
    pub team_away: i32,
    pub name: Option<String>,
}
impl_localized_get!(Game, game_name);
impl_localized_get_by_many!(Game, division);

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
//impl_name_table_name!(League, "league_names", "league");
impl_table_name!(Division, "divisions");
impl_name_table_name!(Division, "division_names", "division");
impl_table_name!(Team, "teams");
impl_name_table_name!(Team, "team_names", "team");
impl_table_name!(Shot, "shots");
impl_table_name!(Game, "games");
impl_name_table_name!(Game, "game_names", "game");
impl_table_name!(Period, "periods");
impl_table_name!(Language, "supported_languages");

#[cfg(test)]
mod tests {
    use crate::languages::SupportedLanguage;
    use crate::model::{
        Division, Game, GamePlayer, Language, League, Player, Shot, TableName, Team,
    };
    use ormx::Table;
    use std::env;
    use strum::{EnumCount, IntoEnumIterator};

    #[test]
    fn db_languages_match_supported_langauges_enum() {
        tokio_test::block_on(async move {
            let pool = db_connect().await;
            let db_langs = Language::all(&pool).await.unwrap();
            assert_eq!(db_langs.len(), SupportedLanguage::COUNT);
            for lang_name in SupportedLanguage::iter() {
                let found = db_langs
                    .iter()
                    .find(|db_lang| db_lang.short_name == format!("{}", lang_name));
                assert!(
                    found.is_some(),
                    "No database language found for variant {lang_name}"
                );
                assert_eq!(found.unwrap().short_name, lang_name.to_string());
            }
        });
    }

    #[test]
    fn test_get_player_from_name() {
        tokio_test::block_on(async move {
            let pool = db_connect().await;
            let player = Player::from_name_case_insensitive(&pool, "tait-hoyem".to_string()).await;
            assert!(player.is_some());
            let player = player.unwrap();
            assert_eq!(player.first_names, "Tait");
        })
    }

    /// A simple function to connect to the database.
    async fn db_connect() -> sqlx::PgPool {
        let db_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL environment variable must be set to run tests.");
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
                    let results = sqlx::query_as::<_, $ret_type>(&format!(
                        "SELECT * FROM {};",
                        <$ret_type as TableName>::TABLE_NAME
                    ))
                    .fetch_all(&pool)
                    .await
                    .unwrap();
                    // check that there is at least one result item
                    assert!(
                        results.len() > 0,
                        "There must be at least one result in the table."
                    );
                });
            }
        };
    }
    generate_select_test!(GamePlayer, selec_game_player);
    generate_select_test!(Player, select_player);
    generate_select_test!(League, select_league);
    generate_select_test!(Division, select_division);
    generate_select_test!(Team, select_team);
    generate_select_test!(Shot, select_shot);
    generate_select_test!(Game, select_game);
    generate_select_test!(Language, select_lang);
}
