mod db;
mod filters;
mod languages;
mod model;
mod traits;
mod views;

macro_rules! other_lang_urls {
  ($lang:expr, $template:ident) => {
    $lang.other_langs().map(move |olang| {
      LangLink {
        name: olang.native_name(),
        href: Into::<Locale>::into($lang)
        .translate(
          $template::URL_KEY_TEMPLATE,
          vec![
            ("lang", olang.into())
          ],
        )
        .expect("Unable to find key {key} in locale {self}.")
      }
    }).collect()
  };
  ($lang:expr, $template:ident, $($k:literal => $v:expr),*) => {
    $lang.other_langs().map(move |olang| {
      LangLink {
        name: olang.native_name(),
        href: Into::<Locale>::into($lang)
        .translate(
          $template::URL_KEY_TEMPLATE,
          hashmap_macro::hashmap![
            "lang" => olang.to_string().into(),
            $($k => $v.into()),*
          ],
        )
        .expect("Unable to find key {key} in locale {self}.")
      }
    }).collect()
  };
}
macro_rules! impl_url_gen {
  ($struct:ident, $($name:ident: $ty:ty),*) => {
    impl $struct<'_> {
      #[allow(dead_code)]
      fn lang_link(lang: SupportedLanguage, $($name: $ty),*) -> LangLink {
        LangLink {
          name: lang.native_name(),
          href: Into::<Locale>::into(lang)
          .translate(
            Self::URL_KEY_TEMPLATE,
            hashmap_macro::hashmap![
              "lang" => lang.into(),
              $(stringify!($name) => $name.into()),*
            ],
          )
          .expect("Unable to find key {key} in locale {self}.")
        }
      }
    }
    #[cfg(test)]
    #[rename_item::rename(case="snake")]
    #[rename::rename_mod(prepend="test_")]
    mod $struct {
      use crate::SupportedLanguage;
      use crate::$struct;
      #[test]
      fn test_lang_link_types() {
        println!("{:?}", $struct::lang_link(SupportedLanguage::English, 0));
      }
    }
  }
}

use static_assertions::assert_impl_all;
use traits::TemplateUrl;
#[macro_use]
extern crate ibihf_macros;
use askama::i18n::{langid, Locale};
askama::i18n::load!(LOCALES);

use crate::model::{Division, Game, Language, League, Player};
use languages::{LangLink, SupportedLanguage};
use views::{GoalDetails, IihfStatsI64, PlayerStats, ShotDetails, TeamStats};

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use ormx::Table;
use sqlx::{Pool, Postgres};
use std::net::SocketAddr;
use std::sync::Arc;

const VERSION: &str = "0.4.0-beta";

#[derive(Template, TemplateUrl)]
#[template(path = "language_list.html")]
struct LanguageListTemplate<'a> {
    #[locale]
    pub loc: Locale<'a>,
    pub lang_links: Vec<LangLink>,
    pub lang: SupportedLanguage,
    pub languages: Vec<Language>,
}

#[derive(Template)]
#[template(path = "partials/box_score_table.html")]
struct BoxScoreTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    goals: Vec<GoalDetails>,
    lang: SupportedLanguage,
}

#[derive(Template)]
#[template(path = "partials/individual_game_points_table.html")]
struct IndividualGamePointsTableTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    players: Vec<PlayerStats>,
}

#[derive(Template)]
#[template(path = "partials/team_stats_table.html")]
struct TeamGameStatsTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    teams: Vec<TeamStats>,
}

#[derive(Template, TemplateUrl)]
#[urls(url_key = "league_url", url_key_template = "league_url_tmpl")]
#[template(path = "division_list.html")]
struct DivisionListTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    lang_links: Vec<LangLink>,
    league: League,
    divisions: Vec<Division>,
    lang: SupportedLanguage,
}
impl_url_gen!(DivisionListTemplate, id: i32);
assert_impl_all!(DivisionListTemplate: TemplateUrl);

#[derive(Template, TemplateUrl)]
#[urls(url_key = "root_url", url_key_template = "root_url_tmpl")]
#[template(path = "league_list.html")]
struct LeagueListTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    lang_links: Vec<LangLink>,
    lang: SupportedLanguage,
    leagues: Vec<League>,
}
impl_url_gen!(LeagueListTemplate, id: i32);
assert_impl_all!(LeagueListTemplate: TemplateUrl);

#[derive(Template)]
#[template(path = "partials/iihf_team_stats_table.html")]
struct IihfTeamStatsTableTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    iihf_stats: Vec<IihfStatsI64>,
}

#[derive(Template, TemplateUrl)]
#[urls(url_key = "division_url", url_key_template = "division_url_tmpl")]
#[template(path = "game_list.html")]
struct GameListTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    lang_links: Vec<LangLink>,
    division: Division,
    iihf_team_stats_table: IihfTeamStatsTableTemplate<'a>,
    games: Vec<Game>,
    lang: SupportedLanguage,
}
impl_url_gen!(GameListTemplate, id: i32);
assert_impl_all!(GameListTemplate: TemplateUrl);

#[derive(Template)]
#[template(path = "partials/play_by_play_table.html")]
struct ShotsTableTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    shots: Vec<ShotDetails>,
    lang: SupportedLanguage,
}

#[derive(Template, TemplateUrl)]
#[urls(url_key = "game_url", url_key_template = "game_url_tmpl")]
#[template(path = "game_score_page.html")]
struct GameScorePageTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    lang_links: Vec<LangLink>,
    game: Game,
    division: Division,
    box_score: BoxScoreTemplate<'a>,
    team_stats: TeamGameStatsTemplate<'a>,
    individual_stats: IndividualGamePointsTableTemplate<'a>,
    play_by_play: ShotsTableTemplate<'a>,
    lang: SupportedLanguage,
}
impl_url_gen!(GameScorePageTemplate, id: i32);
assert_impl_all!(GameScorePageTemplate: TemplateUrl);

#[derive(Template, TemplateUrl)]
#[urls(url_key = "player_url", url_key_template = "player_url_tmpl")]
#[template(path = "player_page.html")]
pub struct PlayerPageTemplate<'a> {
    #[locale]
    locale: Locale<'a>,
    lang_links: Vec<LangLink>,
    player: Player,
    league: League,
    league_stats: PlayerStats,
    lifetime_stats: PlayerStats,
    lang: SupportedLanguage,
}
assert_impl_all!(PlayerPageTemplate: TemplateUrl);

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
        .route(
            &SupportedLanguage::English.lookup(DivisionListTemplate::URL_KEY),
            get(divisions_for_league_html),
        )
        .route(
            &SupportedLanguage::English.lookup(GameListTemplate::URL_KEY),
            get(games_for_division_html),
        )
        .route(
            &SupportedLanguage::English.lookup(GameScorePageTemplate::URL_KEY),
            get(score_for_game_html),
        )
        .route(
            &SupportedLanguage::French.lookup(GameScorePageTemplate::URL_KEY),
            get(score_for_game_html),
        )
        //.route("/:lang/player/:name/", get(player_from_name))
        .with_state(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn language_list(State(server_config): State<ServerState>) -> impl IntoResponse {
    let languages = Language::all(&*server_config.db_pool).await.unwrap();
    let lang_list_tmpl = LanguageListTemplate {
        loc: Locale::new(langid!("en-ca"), &LOCALES),
        lang_links: Vec::new(),
        languages,
        lang: SupportedLanguage::English,
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
    let latest_league = Player::latest_league(&server_config.db_pool, player.id, lang.into())
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
        lang_links: other_lang_urls!(lang, PlayerPageTemplate),
        locale: lang.into(),
        league: latest_league,
        league_stats: latest_league_stats,
        lifetime_stats,
        lang,
    };
    (StatusCode::OK, html)
}

/*
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
*/

async fn league_html(
    State(server_config): State<ServerState>,
    Path(lang): Path<SupportedLanguage>,
) -> impl IntoResponse {
    let leagues = League::all(&*server_config.db_pool, lang.into()).await.unwrap();
    let leagues_template = LeagueListTemplate {
        lang_links: other_lang_urls!(lang, LeagueListTemplate),
        locale: lang.into(),
        leagues,
        lang,
    };
    (StatusCode::OK, leagues_template)
}

async fn divisions_for_league_html(
    State(server_config): State<ServerState>,
    Path((lang, league_id)): Path<(SupportedLanguage, i32)>,
) -> impl IntoResponse {
    let league = League::get(&*server_config.db_pool, league_id, lang.into())
        .await
        .unwrap()
        .unwrap();
    let divisions = Division::by_league(&*server_config.db_pool, league_id, lang.into())
        .await
        .unwrap();
    let html = DivisionListTemplate {
        locale: lang.into(),
        // TODO: add league_id here
        lang_links: other_lang_urls!(lang, DivisionListTemplate, "id" => league.id),
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
    let division = Division::get(&*server_config.db_pool, division_id, lang.into())
        .await
        .unwrap()
        .unwrap();
    let games = Game::by_division(&*server_config.db_pool, division.id, lang.into())
        .await
        .unwrap();
    let iihf_stats = division.iihf_stats(&*server_config.db_pool, lang.into()).await.unwrap();
    let games_template = GameListTemplate {
        locale: lang.into(),
        lang_links: other_lang_urls!(lang, GameListTemplate, "id" => division_id),
        division,
        iihf_team_stats_table: IihfTeamStatsTableTemplate {
            locale: lang.into(),
            iihf_stats,
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
    let game = Game::get(&*server_config.db_pool, game_id, lang.into())
        .await
        .unwrap().unwrap();
    let division = Division::get(&*server_config.db_pool, game.division, lang.into())
        .await
        .unwrap()
        .unwrap();
    let pbp = game.play_by_play(&server_config.db_pool, lang.into()).await.unwrap();
    let score = game.score(&server_config.db_pool, lang.into()).await.unwrap();
    let score_html = TeamGameStatsTemplate {
        locale: lang.into(),
        teams: score,
    };
    let goal_details = game.box_score(&server_config.db_pool).await.unwrap();
    let goal_details_html = IndividualGamePointsTableTemplate {
        locale: lang.into(),
        players: goal_details,
    };
    let box_score = game.goals(&server_config.db_pool, lang.into()).await.unwrap();
    let box_score_html = BoxScoreTemplate {
        locale: lang.into(),
        goals: box_score,
        lang,
    };
    let pbp_html = ShotsTableTemplate {
        locale: lang.into(),
        shots: pbp,
        lang
    };
    let game_template = GameScorePageTemplate {
        locale: lang.into(),
        lang_links: other_lang_urls!(lang, GameScorePageTemplate, "id" => game_id),
        division,
        game,
        box_score: box_score_html,
        team_stats: score_html,
        individual_stats: goal_details_html,
        play_by_play: pbp_html,
        lang,
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
*/
