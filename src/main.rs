mod db;
mod filters;
mod model;
mod views;
mod languages;

use askama::i18n::{langid, Locale};
askama::i18n::load!(LOCALES);

use crate::model::{Division, Game, GamePlayer, League, Player, Shot, Team, Language};
use views::{GoalDetails, PlayerStats, ShotDetails, TeamStats, IihfStatsI64};
use languages::{
  SupportedLanguage,
  LangLink,
}; 

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use axum_macros::debug_handler;
use ormx::Table;
use sqlx::{Pool, Postgres};
use std::net::SocketAddr;
use std::sync::Arc;

const VERSION: &str = "0.3.3";

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
    years: i32,
}

#[derive(Template)]
#[template(path = "language_list.html")]
struct LanguageListTemplate<'a> {
  #[locale]
  pub loc: Locale<'a>,
  pub url_name: &'a str,
  pub lang_links: Vec<LangLink>,
  pub lang: SupportedLanguage,
  pub languages: Vec<Language>,
}

#[derive(Template)]
#[template(path = "partials/box_score_table.html")]
struct BoxScoreTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    lang: SupportedLanguage,
    goals: Vec<GoalDetails>,
}

#[derive(Template)]
#[template(path = "partials/individual_game_points_table.html")]
struct IndividualGamePointsTableTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    lang: SupportedLanguage,
    players: Vec<PlayerStats>,
}

#[derive(Template)]
#[template(path = "partials/team_stats_table.html")]
struct TeamGameStatsTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    teams: Vec<TeamStats>,
    lang: SupportedLanguage,
}

#[derive(Template)]
#[template(path = "division_list.html")]
struct DivisionListTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    url_name: &'a str,
    lang_links: Vec<LangLink>,
    league: League,
    divisions: Vec<Division>,
    lang: SupportedLanguage,
}

#[derive(Template)]
#[template(path = "league_list.html")]
struct LeagueListTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    url_name: &'a str,
    lang_links: Vec<LangLink>,
    lang: SupportedLanguage,
    leagues: Vec<League>,
    heading: String,
}

#[derive(Template)]
#[template(path="partials/iihf_team_stats_table.html")]
struct IihfTeamStatsTableTemplate<'a> {
  #[locale]
  locale: Locale<'a>,
  lang: SupportedLanguage,
	iihf_stats: Vec<IihfStatsI64>,
}

#[derive(Template)]
#[template(path = "game_list.html")]
struct GameListTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    url_name: &'a str,
    lang_links: Vec<LangLink>,
    division: Division,
		iihf_team_stats_table: IihfTeamStatsTableTemplate<'a>,
    games: Vec<Game>,
    lang: SupportedLanguage,
}

#[derive(Template)]
#[template(path = "partials/play_by_play_table.html")]
struct ShotsTableTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    shots: Vec<ShotDetails>,
    lang: SupportedLanguage,
}

#[derive(Template)]
#[template(path = "game_score_page.html")]
struct GameScorePageTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    url_name: &'a str,
    lang_links: Vec<LangLink>,
    game: Game,
    division: Division,
    box_score: BoxScoreTemplate<'a>,
    team_stats: TeamGameStatsTemplate<'a>,
    individual_stats: IndividualGamePointsTableTemplate<'a>,
    play_by_play: ShotsTableTemplate<'a>,
    lang: SupportedLanguage,
}

#[derive(Template)]
#[template(path = "player_page.html")]
pub struct PlayerPageTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    lang_links: Vec<LangLink>,
    url_name: &'a str,
    player: Player,
    league: League,
    league_stats: PlayerStats,
    lifetime_stats: PlayerStats,
    lang: SupportedLanguage,
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
        .route("/", get(language_list))
        .route("/:lang/", get(league_html))
        .route("/:lang/shots/", get(shots_all))
        .route("/:lang/test/", get(test_template))
        .route(&SupportedLanguage::English.lookup("league_url"), get(divisions_for_league_html))
        .route(&SupportedLanguage::English.lookup("division_url"), get(games_for_division_html))
        .route(&SupportedLanguage::English.lookup("game_url"), get(score_for_game_html))
        .route(&SupportedLanguage::French.lookup("game_url"), get(score_for_game_html))
        .route("/:lang/player/:name/", get(player_from_name))
        .with_state(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn language_list(
  State(server_config): State<ServerState>,
) -> impl IntoResponse {
  let languages = Language::all(&*server_config.db_pool)
    .await
    .unwrap();
  let lang_list_tmpl = LanguageListTemplate {
    loc: Locale::new(langid!("en-ca"), &LOCALES),
    url_name: "root_url_tmpl",
    lang_links: Vec::new(),
    lang: SupportedLanguage::English,
    languages
  };
  (StatusCode::OK, lang_list_tmpl)
}

async fn player_from_name(
    State(server_config): State<ServerState>,
    Path((lang, name)): Path<(SupportedLanguage, String)>,
) -> impl IntoResponse {
    let player = Player::from_name_case_insensitive(&server_config.db_pool, name.clone())
        .await
        .unwrap();
    let latest_league = Player::latest_league(&server_config.db_pool, player.id)
        .await
        .unwrap()
        .unwrap();
    let latest_league_stats =
        League::player_stats(&server_config.db_pool, player.id, latest_league.id)
            .await
            .unwrap();
    let lifetime_stats = Player::lifetime_stats(&server_config.db_pool, player.id)
        .await
        .unwrap();
    let html = PlayerPageTemplate {
        player,
        lang_links: lang.other_langs().map(move |olang| LangLink::from_lang_and_name(olang, &name, "player_url_tmpl")).collect(),
        lang,
        locale: lang.into(),
        url_name: "player_url_tmpl",
        league: latest_league,
        league_stats: latest_league_stats,
        lifetime_stats,
    };
    (StatusCode::OK, html)
}

async fn test_template<'a>() -> HelloTemplate<'a> {
    HelloTemplate {
        name: "Tait",
        years: 24,
    }
}

macro_rules! get_all {
    ($crud_struct:ident, $func_name:ident) => {
        #[debug_handler]
        async fn $func_name(State(server_config): State<ServerState>) -> impl IntoResponse {
            let cruder = $crud_struct::all(&*server_config.db_pool).await.unwrap();
            (StatusCode::OK, Json(cruder))
        }
    };
}
macro_rules! get_by_id {
    ($crud_struct:ident, $func_name:ident) => {
        #[debug_handler]
        async fn $func_name(
            State(server_config): State<ServerState>,
            Path(id): Path<i32>,
        ) -> impl IntoResponse {
            let cruder = $crud_struct::get(&*server_config.db_pool, id)
                .await
                .unwrap();
            (StatusCode::OK, Json(cruder))
        }
    };
}

async fn league_html(
    State(server_config): State<ServerState>,
    Path(lang): Path<SupportedLanguage>,
) -> impl IntoResponse {
    let leagues = League::all(&*server_config.db_pool).await.unwrap();
    let heading = match lang {
        SupportedLanguage::English => "IBIHF Leagues",
        SupportedLanguage::French => "League de FIDHS",
    }
    .to_string();
    let leagues_template = LeagueListTemplate {
        lang_links: lang.other_langs().map(move |olang| LangLink::from_lang(olang, "root_url_tmpl")).collect(),
        url_name: "root_url_tmpl",
        locale: lang.into(),
        leagues,
        heading,
        lang,
    };
    (StatusCode::OK, leagues_template)
}

async fn divisions_for_league_html(
    State(server_config): State<ServerState>,
    Path((lang, league_id)): Path<(SupportedLanguage, i32)>,
) -> impl IntoResponse {
    let league = League::get(&*server_config.db_pool, league_id)
        .await
        .unwrap();
    let divisions = Division::by_league(&*server_config.db_pool, league_id)
        .await
        .unwrap();
    let html = DivisionListTemplate {
        locale: lang.into(),
        lang_links: lang.other_langs().map(move |olang| LangLink::from_lang_and_id(olang, league_id, "league_url_tmpl")).collect(),
        url_name: "league_url_tmpl",
        league,
        divisions,
        lang,
    };
    (StatusCode::OK, html)
}

async fn games_for_division_html(
    State(server_config): State<ServerState>,
    Path((lang, division_id)): Path<(SupportedLanguage, i32)>,
) -> impl IntoResponse {
    let division = Division::get(&*server_config.db_pool, division_id)
        .await
        .unwrap();
    let games = Game::by_division(&*server_config.db_pool, division.id)
        .await
        .unwrap();
		let iihf_stats = division.iihf_stats(&*server_config.db_pool)
			.await
			.unwrap();
    let games_template = GameListTemplate {
        locale: lang.into(),
        lang_links: lang.other_langs().map(move |olang| LangLink::from_lang_and_id(olang, division_id, "division_url_tmpl")).collect(),
        url_name: "division_url_tmpl",
        division,
				iihf_team_stats_table: IihfTeamStatsTableTemplate {
          locale: lang.into(),
					iihf_stats,
          lang,
				},
        games,
        lang,
    };
    (StatusCode::OK, games_template)
}
async fn score_for_game_html(
    State(server_config): State<ServerState>,
    Path((lang, game_id)): Path<(SupportedLanguage, i32)>,
) -> impl IntoResponse {
    let game = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id = $1;")
        .bind(game_id)
        .fetch_one(&*server_config.db_pool)
        .await
        .unwrap();
    let division = Division::get(&*server_config.db_pool, game.division)
        .await
        .unwrap();
    let pbp = game.play_by_play(&server_config.db_pool)
        .await
        .unwrap();
    let score = game.score(&server_config.db_pool).await.unwrap();
    let score_html = TeamGameStatsTemplate { locale: lang.into(), teams: score, lang };
    let goal_details = game.box_score(&server_config.db_pool)
        .await
        .unwrap();
    let goal_details_html = IndividualGamePointsTableTemplate {
        locale: lang.into(),
        players: goal_details,
        lang,
    };
    let box_score = game.goals(&server_config.db_pool).await.unwrap();
    let box_score_html = BoxScoreTemplate {
        locale: lang.into(),
        goals: box_score,
        lang,
    };
    let pbp_html = ShotsTableTemplate { locale: lang.into(), shots: pbp, lang };
    let game_template = GameScorePageTemplate {
        locale: lang.into(),
        lang_links: lang.other_langs().map(move |olang| LangLink::from_lang_and_id(olang, game_id, "game_url_tmpl")).collect(),
        url_name: "game_url_tmpl",
        division,
        game,
        lang,
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
    };
}

impl_all_query_types!(GamePlayer, game_player_all, game_player_id);
impl_all_query_types!(Player, player_all, player_id);
impl_all_query_types!(Team, team_all, team_id);
impl_all_query_types!(Shot, shots_all, shots_id);
impl_all_query_types!(Division, division_all, division_id);
impl_all_query_types!(League, league_all, league_id);
