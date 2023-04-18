use crate::LOCALES;
use askama::i18n::fluent_templates::Loader;
use askama::i18n::FluentValue;
use askama::i18n::{langid, LanguageIdentifier, Locale};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumCount, EnumIter, EnumVariantNames};

use derive_more::Display;

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    Display,
    EnumIter,
    EnumCount,
    EnumVariantNames,
    AsRefStr,
    PartialEq,
    Eq,
)]
pub enum SupportedLanguage {
    #[serde(rename = "en-ca")]
    #[display(fmt = "en-ca")]
    English,
    #[serde(rename = "fr-ca")]
    #[display(fmt = "fr-ca")]
    French,
}
impl From<SupportedLanguage> for FluentValue<'_> {
    fn from(n: SupportedLanguage) -> Self {
        n.to_string().into()
    }
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
        LOCALES
            .lookup(&(*self).into(), key)
            .expect("Unable to find key {key} in locale {self}.")
    }
    pub fn other_langs(&self) -> impl Iterator<Item = Self> + '_ {
        Self::iter().filter(move |lang| lang != self)
    }
    pub fn native_name(&self) -> String {
        match self {
            Self::English => "English",
            Self::French => "Français",
        }
        .to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LangLink {
    pub href: String,
    pub name: String,
}
