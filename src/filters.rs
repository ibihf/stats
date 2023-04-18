pub fn seconds_as_time(secs: &i32) -> ::askama::Result<String> {
    let minutes = secs / 60;
    let seconds = secs % 60;
    Ok(format!("{}:{}", minutes, seconds))
}
