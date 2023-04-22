#![allow(dead_code)]

use crate::model::{Division, Game, League, Player};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct TeamStats {
    pub name: String,
    pub goals: i64,
    pub shots: i64,
}

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct IihfStats {
    pub team_name: Option<String>,
    pub team_id: i32,
    pub reg_wins: i32,
    pub reg_losses: i32,
    pub ot_wins: i32,
    pub ot_losses: i32,
    pub ties: i32,
    pub points: i32,
}
#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct IihfStatsI64 {
    pub team_name: Option<String>,
    pub team_id: i32,
    pub reg_wins: i64,
    pub reg_losses: i64,
    pub ot_wins: i64,
    pub ot_losses: i64,
    pub ties: i64,
    pub points: i64,
}
impl From<IihfStats> for IihfStatsI64 {
    fn from(val: IihfStats) -> Self {
        IihfStatsI64 {
            team_name: val.team_name.clone(),
            team_id: val.team_id,
            reg_wins: val.reg_wins.into(),
            reg_losses: val.reg_losses.into(),
            ot_wins: val.ot_wins.into(),
            ot_losses: val.ot_losses.into(),
            ties: val.ties.into(),
            points: val.points.into(),
        }
    }
}

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct IihfPoints {
    pub team_name: String,
    pub team_id: i32,
    pub points: i32,
}

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct Notification {
    pub scorer_first_names: String,
    pub scorer_last_name: String,
    pub scorer_number: i32,
    pub position: String,
    pub scorer_team_name: String,
    pub period_name: String,
    pub period_time_left: i32,
}

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct PlayerStats {
    pub first_names: String,
    pub last_name: String,
    pub goals: i64,
    pub assists: i64,
    pub points: i64,
}

pub async fn game_box_score(pool: &PgPool, game_id: i32) -> Result<Vec<PlayerStats>, sqlx::Error> {
    let query = r#"
SELECT
    COUNT(shots.id) AS points,
    COUNT(CASE WHEN shots.shooter = game_players.id THEN shots.id END) AS goals,
    COUNT(CASE WHEN shots.assistant = game_players.id OR shots.assistant_second = game_players.id THEN shots.id END) AS assists,
    players.first_names,
    players.last_name
FROM game_players
JOIN players
    ON game_players.player = players.id
LEFT JOIN shots
    ON shots.goal=true
    AND (shots.shooter=game_players.id
    OR shots.assistant=game_players.id
    OR shots.assistant_second=game_players.id)
WHERE game_players.game=$1
GROUP BY
    game_players.id,
    players.last_name,
    players.first_names
HAVING COUNT(shots.id) > 0
ORDER BY
    points DESC,
    goals DESC;
"#;
    sqlx::query_as::<_, PlayerStats>(query)
        .bind(game_id)
        .fetch_all(pool)
        .await
}
pub async fn game_goals(
    pool: &PgPool,
    game_id: i32,
    lang: i32,
) -> Result<Vec<GoalDetails>, sqlx::Error> {
    sqlx::query_as::<_, GoalDetails>(
        r#"
  SELECT 
    shots.shooter AS player_id,
    shots.assistant AS first_assist_id,
    shots.assistant_second AS second_assist_id,
    players.first_names AS player_first_names,
    players.last_name AS player_last_name,
    p_assist.first_names AS first_assist_first_names,
    p_assist.last_name AS first_assist_last_name,
    p_assist_second.first_names AS second_assist_first_names,
    p_assist_second.last_name AS second_assist_last_name,
    game_players.player_number AS player_number,
    gp_assist.player_number AS first_assist_number,
    gp_assist_second.player_number AS second_assist_number,
    team_name(teams.id, $1) AS team_name,
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
  "#,
    )
    .bind(game_id)
    .bind(lang)
    .fetch_all(pool)
    .await
}
pub async fn game_iihf_stats(
    pool: &PgPool,
    game_id: i32,
    lang: i32,
) -> Result<Vec<IihfStats>, sqlx::Error> {
    let query = r#"
	SELECT
    teams.id AS team_id,
    team_name(teams.id, $2) AS team_name,
    reg_win(games.id, teams.id) AS reg_wins,
    reg_loss(games.id, teams.id) AS reg_losses,
    ot_win(games.id, teams.id) AS ot_wins,
    ot_loss(games.id, teams.id) AS ot_losses,
    tie(games.id, teams.id) AS ties,
    iihf_points(games.id, teams.id) AS points
	FROM games
  JOIN periods ON periods.game=games.id
  JOIN shots ON shots.period=periods.id
  JOIN game_players ON game_players.id=shots.shooter
  JOIN teams scoring_team
    ON scoring_team.id=game_players.team
  JOIN teams
    ON teams.id=games.team_home
    OR teams.id=games.team_away
 WHERE games.id=4
 GROUP BY teams.id,games.id;
  "#;
    sqlx::query_as::<_, IihfStats>(query)
        .bind(game_id)
        .bind(lang)
        .fetch_all(pool)
        .await
}
/// Returns the number of points using IIHF scoring rules for each team.
/// NOTE: The algorithm used here requires that a 4th period is the "overtime";
/// it does not check if there was only two periods, followed by an overtime.
/// This should be sufficient for most.
pub async fn game_iihf_points(
    pool: &PgPool,
    game_id: i32,
    lang: i32,
) -> Result<Vec<IihfPoints>, sqlx::Error> {
    let query = r#"
  SELECT 
		iihf_points(games.id, teams.id) AS points,
    team_name(teams.id, $1) AS team_name,
		teams.id AS team_id
  FROM games
	JOIN teams
		ON teams.id=games.team_home
		OR teams.id=games.team_away
  WHERE games.id=$1
  ORDER BY points;
  "#;
    sqlx::query_as::<_, IihfPoints>(query)
        .bind(game_id)
        .bind(lang)
        .fetch_all(pool)
        .await
}
/// Returns the number of shots and goals for each team in the game.
pub async fn game_score(
    pool: &PgPool,
    game_id: i32,
    lang: i32,
) -> Result<Vec<TeamStats>, sqlx::Error> {
    let query = r#"
  SELECT 
    COUNT(CASE WHEN shots.goal = true THEN shots.id END) AS goals,
    COUNT(shots.id) AS shots,
    team_name(teams.id, $2) AS name
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
        .bind(lang)
        .fetch_all(pool)
        .await
}
pub async fn game_play_by_play(
    pool: &PgPool,
    game_id: i32,
    lang: i32,
) -> Result<Vec<ShotDetails>, sqlx::Error> {
    sqlx::query_as::<_, ShotDetails>(
        r#"
SELECT 
  shots.shooter AS player_id,
  shots.assistant AS first_assist_id,
  shots.assistant_second AS second_assist_id,
  shots.goal AS is_goal,
  players.first_names AS player_first_names,
  players.last_name AS player_last_name,
  p_assistant.first_names AS first_assist_first_names,
  p_assistant.last_name AS first_assist_last_name,
  p_assistant_second.first_names AS second_assist_first_names,
  p_assistant_second.last_name AS second_assist_last_name,
  game_players.player_number AS player_number,
  gp_assistant.player_number AS first_assist_number,
  gp_assistant_second.player_number AS second_assist_number,
  team_name(teams.id, $2) AS team_name,
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
"#,
    )
    .bind(game_id)
    .bind(lang)
    .fetch_all(pool)
    .await
}

impl Game {
    pub async fn score(&self, pool: &PgPool, lang: i32) -> Result<Vec<TeamStats>, sqlx::Error> {
        game_score(pool, self.id, lang).await
    }
    pub async fn box_score(&self, pool: &PgPool) -> Result<Vec<PlayerStats>, sqlx::Error> {
        game_box_score(pool, self.id).await
    }
    pub async fn iihf_points(
        &self,
        pool: &PgPool,
        lang: i32,
    ) -> Result<Vec<IihfPoints>, sqlx::Error> {
        game_iihf_points(pool, self.id, lang).await
    }
    pub async fn iihf_stats(
        &self,
        pool: &PgPool,
        lang: i32,
    ) -> Result<Vec<IihfStats>, sqlx::Error> {
        game_iihf_stats(pool, self.id, lang).await
    }
    pub async fn goals(&self, pool: &PgPool, lang: i32) -> Result<Vec<GoalDetails>, sqlx::Error> {
        game_goals(pool, self.id, lang).await
    }
    pub async fn play_by_play(
        &self,
        pool: &PgPool,
        lang: i32,
    ) -> Result<Vec<ShotDetails>, sqlx::Error> {
        game_play_by_play(pool, self.id, lang).await
    }
}

pub async fn division_iihf_stats(
    pool: &PgPool,
    division_id: i32,
    lang: i32,
) -> Result<Vec<IihfStatsI64>, sqlx::Error> {
    sqlx::query_as!(
      IihfStatsI64,
        r#"
SELECT
  SUM(points) AS "points!",
  SUM(reg_wins) AS "reg_wins!",
  SUM(reg_losses) AS "reg_losses!",
  SUM(ot_wins) AS "ot_wins!",
  SUM(ot_losses) AS "ot_losses!",
  SUM(ties) AS "ties!",
  team_name(team_id, $2) AS team_name,
  team_id AS "team_id!"
FROM team_points_view
WHERE division_id=$1
GROUP BY team_id;
--WITH team_name AS (
--  SELECT
--    teams.id AS team_id,
--    -- max will get the first matching string; technically it will always get the string that that has the maximum value based on the locale, ignoring nulls.
--    COALESCE(
--      MAX(localized_name.name),
--      MAX(default_name.name),
--      MAX(any_name.name)
--    ) AS team_name,
--    -- this is to get the language id of the team name; although not strictly necessary, since we use MIN(...), then ORDER BY it later on, we prioritize languages that have been added earlier, making this table deterministic
--    COALESCE(
--      MIN(localized_name.language),
--      MIN(default_name.language),
--      MIN(any_name.language)
--    ) AS name_language
--  FROM teams
--  LEFT JOIN team_names localized_name ON localized_name.team = teams.id AND localized_name.language = $2
--  LEFT JOIN team_names default_name ON default_name.team = teams.id AND default_name.language = 1
--  LEFT JOIN team_names any_name ON any_name.team = teams.id
--  GROUP BY teams.id
--  ORDER BY name_language
--)
--SELECT
--	SUM(reg_win(games.id, teams.id)) AS reg_wins,
--	SUM(reg_loss(games.id, teams.id)) AS reg_losses,
--	SUM(ot_win(games.id, teams.id)) AS ot_wins,
--	SUM(ot_loss(games.id, teams.id)) AS ot_losses,
--	SUM(tie(games.id, teams.id)) AS ties,
--	SUM(iihf_points(games.id, teams.id)) AS points,
--	teams.id AS team_id,
--  team_name.team_name
--FROM
--	games
--JOIN teams ON teams.id=games.team_home OR teams.id=games.team_away
--JOIN team_name ON team_name.team_id=teams.id
--WHERE games.division=$1
--GROUP BY
--	teams.id,
--  team_name.team_name
--ORDER BY
--	points DESC;

--SELECT DISTINCT ON (teams.id)
--	SUM(reg_win(games.id, teams.id)) AS reg_wins,
--	SUM(reg_loss(games.id, teams.id)) AS reg_losses,
--	SUM(ot_win(games.id, teams.id)) AS ot_wins,
--	SUM(ot_loss(games.id, teams.id)) AS ot_losses,
--	SUM(tie(games.id, teams.id)) AS ties,
--	SUM(iihf_points(games.id, teams.id)) AS points,
--	teams.id AS team_id,
--	COALESCE(
--    localized_name.name,
--    default_name.name,
--    any_name.name
--  ) AS team_name
--FROM
--	games
--JOIN teams ON teams.id=games.team_home OR teams.id=games.team_away
--LEFT JOIN team_names localized_name
--       ON localized_name.team = teams.id
--      AND localized_name.language = $2
--LEFT JOIN team_names default_name
--       ON default_name.team = teams.id
--      AND default_name.language = 1
--LEFT JOIN team_names any_name
--       ON any_name.team = teams.id
--WHERE games.division=$1
--GROUP BY
--	teams.id,
--  localized_name.name,
--  default_name.name,
--  any_name.name
--ORDER BY
--  teams.id,
--	points DESC;
		"#,
    division_id, lang
    )
    .fetch_all(pool)
    .await
}

impl Division {
    pub async fn iihf_stats(
        &self,
        pool: &PgPool,
        lang: i32,
    ) -> Result<Vec<IihfStatsI64>, sqlx::Error> {
        division_iihf_stats(pool, self.id, lang).await
    }
}

impl Player {
    pub async fn latest_league(
        pool: &PgPool,
        id: i32,
        lang: i32,
    ) -> Result<Option<League>, sqlx::Error> {
        let query = r#"
  SELECT leagues.*,team_name(teams.id, $2) AS name
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
            .bind(lang)
            .fetch_optional(pool)
            .await
    }
    pub async fn latest_stats(
        pool: &PgPool,
        id: i32,
        lang: i32,
    ) -> Result<Vec<GoalDetails>, sqlx::Error> {
        let query = r#"
  SELECT 
    players.id AS player_id,
    p_assist.id AS first_assist_id,
    p_assist_second.id AS second_assist_id,
    players.first_names AS player_first_names,
    players.last_name AS player_last_name,
    p_assist.first_names AS first_assist_first_names,
    p_assist.last_name AS first_assist_last_name,
    p_assist_second.first_names AS second_assist_first_names,
    p_assist_second.last_name AS second_assist_last_name,
    game_players.player_number AS player_number,
    gp_assist.player_number AS first_assist_number,
    gp_assist_second.player_number AS second_assist_number,
    team_name(teams.id, $2) AS team_name,
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
        sqlx::query_as::<_, GoalDetails>(query)
            .bind(id)
            .bind(lang)
            .fetch_all(pool)
            .await
    }
    pub async fn lifetime_stats(pool: &PgPool, id: i32) -> Result<PlayerStats, sqlx::Error> {
        let query = r#"
  SELECT
    COUNT(goals) AS goals,
    COUNT(assists) AS assists,
    COUNT(points) AS points,
    players.first_names AS first_names,
    players.last_name AS last_name
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
  players.first_names AS first_names,
  players.last_name AS last_name
FROM game_players
JOIN players ON game_players.player = players.id
LEFT JOIN shots
  ON shots.goal=true
 AND (shots.shooter=game_players.id
  OR shots.assistant=game_players.id
  OR shots.assistant_second=game_players.id)
GROUP BY
  game_players.id,
  players.first_names,
  players.last_name
ORDER BY
  points DESC,
  goals DESC;
"#;
    sqlx::query_as::<_, PlayerStats>(query)
        .fetch_all(&pool)
        .await
}

impl League {
    pub async fn player_stats(
        pool: &PgPool,
        player_id: i32,
        league_id: i32,
    ) -> Result<PlayerStats, sqlx::Error> {
        let query = r#"
  SELECT
    COUNT(goals.id) AS goals,
    COUNT(assists.id) AS assists,
    COUNT(points.id) AS points,
    players.first_names AS first_names,
    players.last_name AS last_name
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
    pub player_first_names: String,
    pub player_last_name: String,
    pub player_number: i32,
    pub team_name: String,
    pub team_id: i32,
    pub time_remaining: i32,
    pub period_id: i32,
    pub period_short_name: String,
    pub first_assist_first_names: Option<String>,
    pub first_assist_last_name: Option<String>,
    pub first_assist_number: Option<i32>,
    pub first_assist_id: Option<i32>,
    pub second_assist_first_names: Option<String>,
    pub second_assist_last_name: Option<String>,
    pub second_assist_id: Option<i32>,
    pub second_assist_number: Option<i32>,
}

#[derive(FromRow, Deserialize, Serialize, Debug)]
pub struct ShotDetails {
    pub player_id: i32,
    pub player_first_names: String,
    pub player_last_name: String,
    pub player_number: i32,
    pub team_name: String,
    pub team_id: i32,
    pub is_goal: bool,
    pub time_remaining: i32,
    pub period_short_name: String,
    pub first_assist_first_names: Option<String>,
    pub first_assist_last_name: Option<String>,
    pub first_assist_number: Option<i32>,
    pub first_assist_id: Option<i32>,
    pub second_assist_first_names: Option<String>,
    pub second_assist_last_name: Option<String>,
    pub second_assist_id: Option<i32>,
    pub second_assist_number: Option<i32>,
}

#[cfg(test)]
mod tests {
    use crate::languages::SupportedLanguage;
    use crate::model::{Game, League, Player};
    use crate::views::{
        division_iihf_stats, game_box_score, game_goals, game_iihf_points, game_iihf_stats,
        game_play_by_play, game_score, get_player_stats_overview, Notification,
    };
    use ormx::Table;
    use std::env;

    #[test]
    fn check_play_by_play() {
        tokio_test::block_on(async move {
            let pool = db_connect().await;
            let pbp = game_play_by_play(&pool, 3, SupportedLanguage::English.into())
                .await
                .unwrap();
        })
    }

    #[test]
    fn get_latest_stats_of_player() {
        tokio_test::block_on(async move {
            let pool = db_connect().await;
            let player = Player::get(&pool, 2).await.unwrap();
            let latest = Player::latest_stats(&pool, player.id, SupportedLanguage::English.into())
                .await
                .unwrap();
        })
    }

    #[test]
    fn check_league_player_stats() {
        tokio_test::block_on(async move {
            let pool = db_connect().await;
            let league = League::get(&pool, 1, SupportedLanguage::English.into())
                .await
                .unwrap()
                .unwrap();
            let player = Player::get(&pool, 2).await.unwrap();
            let stats = League::player_stats(&pool, player.id, league.id)
                .await
                .unwrap();
            assert_eq!(stats.last_name, "Scanlon");
        })
    }

    #[test]
    fn check_latest_league_for_player() {
        tokio_test::block_on(async move {
            let pool = db_connect().await;
            let player = Player::get(&pool, 5).await.unwrap();
            let league = Player::latest_league(&pool, player.id, SupportedLanguage::English.into())
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
            let scores = game_goals(&pool, 3, SupportedLanguage::English.into())
                .await
                .unwrap();
            println!("{scores:?}");
        })
    }

    #[test]
    fn check_box_score_from_game() {
        tokio_test::block_on(async move {
            let pool = db_connect().await;
            let scores = game_box_score(&pool, 4).await.unwrap();
            println!("{scores:?}");
            let second_top_scorer = scores.get(1).unwrap();
            assert_eq!(second_top_scorer.last_name, "Foulds");
            assert_eq!(second_top_scorer.goals, 1, "Allysa should have 1 goal..");
            assert_eq!(
                second_top_scorer.assists, 2,
                "Allyssa should have 2 assists."
            );
            assert_eq!(second_top_scorer.points, 3, "Allysa should have 3 points.");
            assert_eq!(
                scores.len(),
                8,
                "Players which did not receive any points should not be in the box score."
            );
        })
    }

    #[test]
    fn check_division_iihf_stats() {
        tokio_test::block_on(async move {
            let pool = db_connect().await;
            let score = division_iihf_stats(&pool, 1, SupportedLanguage::English.into())
                .await
                .unwrap();
            let team_1 = score.get(0).unwrap();
            let team_2 = score.get(1).unwrap();
            assert_eq!(score.len(), 2, "Too many teams selected.");
            assert_eq!(team_1.points, 10, "Top team should have 10 points");
            assert_eq!(
                team_1.team_name.as_ref().unwrap(),
                "Bullseye",
                "Top team should be bullseye"
            );
            assert_eq!(
                team_1.reg_losses, 0,
                "The bullseye should have no regulation losses"
            );
            assert_eq!(team_1.ties, 2, "There should be two ties for the bullsye");
            assert_eq!(
                team_2.team_name.as_ref().unwrap(),
                "See Cats",
                "The second-place team should be the see cats"
            );
            assert_eq!(
                team_2.points, 4,
                "The second-place team should have four points"
            );
        })
    }

    #[test]
    fn check_iihf_stats() {
        tokio_test::block_on(async move {
            let pool = db_connect().await;
            let score = game_iihf_stats(&pool, 4, SupportedLanguage::English.into())
                .await
                .unwrap();
            let team_1 = score.get(0).unwrap();
            assert_eq!(team_1.points, 2);
            assert_eq!(team_1.team_name.as_ref().unwrap(), "Bullseye");
            assert_eq!(team_1.reg_losses, 0);
            assert_eq!(team_1.ties, 1);
            assert_eq!(score.get(1).unwrap().points, 2);
        })
    }

    #[test]
    fn check_iihf_points() {
        tokio_test::block_on(async move {
            let pool = db_connect().await;
            let score = game_iihf_points(&pool, 4, SupportedLanguage::English.into())
                .await
                .unwrap();
            assert_eq!(score.get(0).unwrap().points, 2);
            assert_eq!(score.get(0).unwrap().team_name, "Bullseye");
            assert_eq!(score.get(1).unwrap().points, 2);
        })
    }

    #[test]
    fn check_game_score() {
        tokio_test::block_on(async move {
            let pool = db_connect().await;
            let score = game_score(&pool, 1, SupportedLanguage::English.into())
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
            let lifetime_stats = Player::lifetime_stats(&pool, 5).await.unwrap();
        })
    }

    #[test]
    fn check_notification_query() {
        tokio_test::block_on(async move {
            let pool = db_connect().await;
            let query = r#"
SELECT
  team_name(teams.id, $1) AS scorer_team_name,
  players.first_names AS scorer_first_names,
  players.last_name AS scorer_last_name,
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
                .bind(1)
                .fetch_one(&pool)
                .await
                .unwrap();
            let minutes = result.period_time_left / 60;
            let seconds = result.period_time_left % 60;
            println!(
                "{0} {1} player #{3} {2} has scored! Time of the goal: {4}:{5} in the {6}",
                result.scorer_team_name,
                result.position,
                result.scorer_last_name,
                result.scorer_number,
                minutes,
                seconds,
                result.period_name
            );
        });
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
}
