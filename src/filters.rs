// We must always wrap a return type of a filter in a Result.
// This sometimes triggers a clippy warning.
#![allow(clippy::unnecessary_wraps)]
// We must always take references, even when it's not technically the fastest thing to do.
// This sometimes also causes a clippy warning.
#![allow(clippy::trivially_copy_pass_by_ref)]
use crate::{GoalDetails, Player, ShotDetails, SupportedLanguage};

pub fn seconds_as_time(secs: &i32) -> ::askama::Result<String> {
    let minutes = secs / 60;
    let seconds = secs % 60;
    Ok(format!("{minutes}:{seconds}"))
}
pub fn player_name(player: &Player) -> ::askama::Result<String> {
    Ok(format!(
        "{} {}",
        initials(&player.first_names)?,
        &player.last_name
    ))
}
pub fn goal_player_name(goal: &GoalDetails) -> ::askama::Result<String> {
    Ok(format!(
        "{} {}",
        initials(&goal.player_first_names)?,
        &goal.player_last_name
    ))
}
pub fn goal_assist_name(goal: &GoalDetails, lang: &SupportedLanguage) -> ::askama::Result<String> {
    let Some(ref f_names) = goal.second_assist_first_names else {
		return Ok(lang.lookup("not-applicable"));
	};
    let Some(ref l_name) = goal.second_assist_last_name else {
		return Ok(lang.lookup("not-applicable"));
	};
    Ok(format!("{f_names} {l_name}"))
}
pub fn shot_assist_name(shot: &ShotDetails, lang: &SupportedLanguage) -> ::askama::Result<String> {
	if !shot.is_goal {
		return Ok(lang.lookup("not-applicable"));
	}
    let Some(ref f_names) = shot.first_assist_first_names else {
		return Ok(lang.lookup("unassisted"));
	};
    let Some(ref l_name) = shot.first_assist_last_name else {
		return Ok(lang.lookup("unassisted"));
	};
    Ok(format!("{f_names} {l_name}"))
}
pub fn goal_second_assist_name(
    goal: &GoalDetails,
    lang: &SupportedLanguage,
) -> ::askama::Result<String> {
    let Some(ref f_names) = goal.second_assist_first_names else {
		return Ok(lang.lookup("not-applicable"));
	};
    let Some(ref l_name) = goal.second_assist_last_name else {
		return Ok(lang.lookup("not-applicable"));
	};
    Ok(format!("{f_names} {l_name}"))
}
pub fn shot_second_assist_name(
    shot: &ShotDetails,
    lang: &SupportedLanguage,
) -> ::askama::Result<String> {
	if !shot.is_goal ||
		shot.second_assist_id.is_none() {
		return Ok(lang.lookup("not-applicable"));
	}
    let Some(ref f_names) = shot.second_assist_first_names else {
		return Ok(lang.lookup("unassisted"));
	};
    let Some(ref l_name) = shot.second_assist_last_name else {
		return Ok(lang.lookup("unassisted"));
	};
	Ok(format!("{f_names} {l_name}"))
}
pub fn shot_player_name(shot: &ShotDetails) -> ::askama::Result<String> {
    Ok(format!(
        "{} {}",
        initials(&shot.player_first_names)?,
        &shot.player_last_name
    ))
}
pub fn initials(first_names: &str) -> ::askama::Result<String> {
    Ok(format!(
        "{}.",
        first_names
            .split_whitespace()
            .map(|name| &name[0..1])
            .collect::<Vec<_>>()
            .join(".")
    ))
}
pub fn nullable<T: std::fmt::Display>(ot: &Option<T>) -> ::askama::Result<String> {
    match ot {
        Some(t) => Ok(format!("{t}")),
        None => Ok("NULL".to_string()),
    }
}
