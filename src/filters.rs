pub fn seconds_as_time(secs: &i32) -> ::askama::Result<String> {
    let minutes = secs / 60;
    let seconds = secs % 60;
    Ok(format!("{}:{}", minutes, seconds))
}
pub fn nullable<T: std::fmt::Display>(ot: &Option<T>) -> ::askama::Result<String> {
  match ot {
    Some(t) => Ok(format!("{}", t)),
    None => Ok("NULL".to_string())
  }
}
