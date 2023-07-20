use crate::{SupportedLanguage, Locale, VERSION};
use axum::Form;
use askama_axum::Template;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Login {
  username: String,
  password: String,
}

impl std::fmt::Display for Login {
  fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(formatter, r#"
  <form method="POST">
    <label for="uname">Username</label>
    <input id="uname" type="text" name="username"/>
    <br/>
    <label for="pass">Password</label>
    <input id="pass" type="password" name="password"/>
    <input type="submit"/>
  </forn> 
"#);
    Ok(())
  }
}
