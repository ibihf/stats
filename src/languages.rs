use serde::{
  Serialize,
  Deserialize,
};
use strum_macros::{
  EnumIter,
  EnumVariantNames,
  EnumCount,
};
  
use derive_more::Display;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Display, EnumIter, EnumCount, EnumVariantNames)]
pub enum SupportedLanguage {
  #[serde(rename="en-ca")]
  #[display(fmt="en-ca")]
  English,
  #[serde(rename="fr-ca")]
  #[display(fmt="fr-ca")]
  French,
}
