use crate::LOCALES;
use askama::i18n::fluent_templates::Loader;
use askama::i18n::FluentValue;
use askama::i18n::{langid, LanguageIdentifier, Locale};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumCount, EnumIter, EnumVariantNames};

use derive_more::Display;

#[derive(
    sqlx::Type,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    Hash,
    Display,
    EnumIter,
    EnumCount,
    EnumVariantNames,
    AsRefStr,
    PartialEq,
    Eq,
)]
#[repr(i32)]
pub enum SupportedLanguage {
    #[serde(rename = "en-ca")]
    #[display(fmt = "en-ca")]
    English = 1,
    #[serde(rename = "fr-ca")]
    #[display(fmt = "fr-ca")]
    French = 2,
}
impl From<SupportedLanguage> for i32 {
  fn from(lang: SupportedLanguage) -> Self {
    match lang {
      SupportedLanguage::English => 1,
      SupportedLanguage::French => 2,
		}
	}
}
impl From<SupportedLanguage> for FluentValue<'_> {
    fn from(n: SupportedLanguage) -> Self {
        n.to_string().into()
    }
}
impl From<SupportedLanguage> for LanguageIdentifier {
    fn from(lang: SupportedLanguage) -> LanguageIdentifier {
        match lang {
            SupportedLanguage::English => langid!("en-ca"),
            SupportedLanguage::French => langid!("fr-ca"),
        }
    }
}
impl<'a> From<SupportedLanguage> for Locale<'a> {
    fn from(lang: SupportedLanguage) -> Self {
        Locale::new(lang.into(), &LOCALES)
		}
}
impl SupportedLanguage {
    pub fn lookup(self, key: &str) -> String {
        LOCALES
            .lookup(&self.into(), key)
            .expect("Unable to find key {key} in locale {self}.")
    }
    pub fn other_langs(self) -> impl Iterator<Item = Self> + 'static {
        Self::iter().filter(move |lang| lang != &self)
    }
    pub fn native_name(self) -> String {
        match self {
            Self::English => "English",
            Self::French => "Français",
        }
        .to_string()
    }
  pub fn id(self) -> i32 {
    match self {
      Self::English => 1,
      Self::French => 2,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LangLink {
    pub href: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalizedName {
  pub localizations: std::collections::HashMap<SupportedLanguage, String>,
}
impl LocalizedName {
  fn localize(&self, lang: SupportedLanguage) -> Option<String> {
    // first, try to find the proper match for a name
    self.localizations
        .iter()
        .find_map(|(translated_lang, string)| if translated_lang == &lang {
          Some(string.to_string())
        } else {
          None
        })
    // if not found, replace it with ANY localization of the word; this will help when, for example, there is no matching name for a French game, but the game has already been created with the English name.
    // if NO localization is found, then we can still return a None value; but hopefully through database, form, and web server restrictions, we can mostly stop that from happening
        .or_else(|| if self.localizations.is_empty() { 
          Some(self.localizations.values().next().unwrap().to_string())
        } else {
          None
        })
  }
}
