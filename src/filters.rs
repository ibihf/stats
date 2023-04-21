use crate::{
  Player,
  ShotDetails,
  GoalDetails,
  SupportedLanguage,
};

pub fn seconds_as_time(secs: &i32) -> ::askama::Result<String> {
    let minutes = secs / 60;
    let seconds = secs % 60;
    Ok(format!("{}:{}", minutes, seconds))
}
pub fn player_name(player: &Player) -> ::askama::Result<String> {
  Ok(format!("{} {}", initials(&player.first_names)?, &player.last_name))
}
pub fn goal_player_name(goal: &GoalDetails) -> ::askama::Result<String> {
  Ok(format!("{} {}", initials(&goal.player_first_names)?, &goal.player_last_name))
}
pub fn goal_assist_name(goal: &GoalDetails, lang: &SupportedLanguage) -> ::askama::Result<String> {
  let initials = match goal.first_assist_first_names {
    Some(ref f_names) => initials(f_names)?,
    None => return Ok(lang.lookup("not-applicable")),
  };
  let last_name = match goal.first_assist_last_name {
    Some(ref l_name) => l_name,
    None => return Ok(lang.lookup("not-applicable")),
  };
  Ok(format!("{} {}", initials, last_name))
}
pub fn shot_assist_name(goal: &ShotDetails, lang: &SupportedLanguage) -> ::askama::Result<String> {
  let initials = match goal.first_assist_first_names {
    Some(ref f_names) => initials(&f_names)?,
    None => return Ok(lang.lookup("not-applicable")),
  };
  let last_name = match goal.first_assist_last_name {
    Some(ref l_name) => l_name,
    None => return Ok(lang.lookup("not-applicable")),
  };
  Ok(format!("{} {}", initials, last_name))
}
pub fn goal_second_assist_name(goal: &GoalDetails, lang: &SupportedLanguage) -> ::askama::Result<String> {
  let initials = match goal.second_assist_first_names {
    Some(ref f_names) => initials(f_names)?,
    None => return Ok(lang.lookup("not-applicable")),
  };
  let last_name = match goal.second_assist_last_name {
    Some(ref l_name) => l_name,
    None => return Ok(lang.lookup("not-applicable")),
  };
  Ok(format!("{} {}", initials, last_name))
}
pub fn shot_second_assist_name(goal: &ShotDetails, lang: &SupportedLanguage) -> ::askama::Result<String> {
  let initials = match goal.second_assist_first_names {
    Some(ref f_names) => initials(f_names)?,
    None => return Ok(lang.lookup("not-applicable")),
  };
  let last_name = match goal.second_assist_last_name {
    Some(ref l_name) => l_name,
    None => return Ok(lang.lookup("not-applicable")),
  };
  Ok(format!("{} {}", initials, last_name))
}
pub fn shot_player_name(shot: &ShotDetails) -> ::askama::Result<String> {
  Ok(format!("{} {}", initials(&shot.player_first_names)?, &shot.player_last_name))
}
pub fn initials(first_names: &str) -> ::askama::Result<String> {
  Ok(first_names
    .split_whitespace()
    .map(|name| &name[0..1])
    .collect::<Vec<_>>()
    .join("."))
}
pub fn nullable<T: std::fmt::Display>(ot: &Option<T>) -> ::askama::Result<String> {
  match ot {
    Some(t) => Ok(format!("{}", t)),
    None => Ok("NULL".to_string())
  }
}
