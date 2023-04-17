use rust_i18n::t;
use crate::languages::SupportedLanguage;

pub fn seconds_as_time(secs: &i32) -> ::askama::Result<String> {
  let minutes = secs / 60;
  let seconds = secs % 60;
  Ok(format!("{}:{}", minutes, seconds))
}

pub fn intl(key: &str, lang: &SupportedLanguage) -> ::askama::Result<String> {
  Ok(t!(key, locale=&format!("{}", lang)))
}
