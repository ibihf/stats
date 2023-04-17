use crate::LOCALES;
use askama::i18n::{langid, LanguageIdentifier, Locale};
use askama::i18n::fluent_templates::Loader;
use serde::{
  Serialize,
  Deserialize,
};
use strum_macros::{
  EnumIter,
  AsRefStr,
  EnumVariantNames,
  EnumCount,
};
  
use derive_more::Display;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Display, EnumIter, EnumCount, EnumVariantNames, AsRefStr)]
pub enum SupportedLanguage {
  #[serde(rename="en-ca")]
  #[display(fmt="en-ca")]
  English,
  #[serde(rename="fr-ca")]
  #[display(fmt="fr-ca")]
  French,
}
impl Into<LanguageIdentifier> for SupportedLanguage {
  fn into(self) -> LanguageIdentifier {
    match self {
      Self::English => langid!("en-ca"),
      Self::French => langid!("fr-ca"),
    }
  }
}
impl<'a> Into<Locale<'a>> for SupportedLanguage {
  fn into(self) -> Locale<'a> {
    Locale::new(self.into(), &LOCALES)
  }
}
impl SupportedLanguage {
  pub fn lookup(&self, key: &str) -> String {
    LOCALES.lookup(&(*self).into(), key).expect("Unable to find key {key} in locale {self}.")
  }
}
