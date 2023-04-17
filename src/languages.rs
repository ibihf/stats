use std::collections::HashMap;
use crate::LOCALES;
use askama::i18n::{langid, LanguageIdentifier, Locale};
use askama::i18n::fluent_templates::Loader;
use serde::{
  Serialize,
  Deserialize,
};
use strum::{
  IntoEnumIterator,
};
use strum_macros::{
  EnumIter,
  AsRefStr,
  EnumVariantNames,
  EnumCount,
};
  
use derive_more::Display;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Display, EnumIter, EnumCount, EnumVariantNames, AsRefStr, PartialEq, Eq)]
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
  pub fn other_langs(&self) -> impl Iterator<Item=Self> + '_ {
    Self::iter()
      .filter(move |lang| lang != self)
  }
  pub fn native_name(&self) -> String {
    match self {
      Self::English => "English",
      Self::French => "Français"
    }.to_string()
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LangLink {
  pub href: String,
  pub name: String,
}

macro_rules! lang_url_match {
  ($lang:expr, $link_name:expr) => {
      Into::<Locale>::into($lang)
      .translate(
        $link_name,
        hashmap_macro::hashmap! {
          "lang" => $lang.to_string().into()
        },
      )
      .expect("Unable to find key {key} in locale {self}.")
  };
  ($lang:expr, $link_name:expr, $id:expr) => {
      Into::<Locale>::into($lang)
      .translate(
        $link_name,
        hashmap_macro::hashmap! {
          "lang" => $lang.to_string().into(),
          "id" => $id.into()
        },
      )
      .expect("Unable to find key {key} in locale {self}.")
  };
}

// TODO: Genericize this so it can accept any arugments through a impl Iterator<(K, V>) or something similar.
impl LangLink {
  pub fn from_lang(lang: SupportedLanguage, link_name: &str) -> Self {
    Self {
      name: lang.native_name(),
      href: lang_url_match!(lang, link_name),
    }
  }
  pub fn from_lang_and_id(lang: SupportedLanguage, id: i32, link_name: &str) -> Self {
    Self {
      name: lang.native_name(),
      href: lang_url_match!(lang, link_name, id),
    }
  }
  pub fn from_lang_and_name(lang: SupportedLanguage, name: &str, link_name: &str) -> Self {
    Self {
      name: lang.native_name(),
      href: lang_url_match!(lang, link_name, name),
    }
  }
}
