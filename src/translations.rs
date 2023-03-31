use serde::{Serialize, Deserialize};
use strum::{
	EnumIter,
	IntoEnumIterator,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum SupportedLanguage {
	#[serde(rename = "en")]
	English,
	#[serde(rename = "fr")]
	French,
}
impl std::fmt::Display for SupportedLanguage {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let output = match self {
			Self::English => "en",
			Self::French => "fr",
		};
		write!(f, "{}", output)
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq, EnumIter)]
#[serde(rename_all = "camelCase")]
pub enum TranslatedKey {
	UrlGame,
	UrlDivision,
	UrlLeague,
	IbihfLeagues,
	Goals,
	Assists,
	Period,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TranslatedString {
	#[serde(rename = "name")]
	pub key: TranslatedKey,
	#[serde(rename = "$value")]
	pub value: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LanguageStrings {
	#[serde(rename = "$value")]
	pub kvs: Vec<TranslatedString>,
}

/// Verify that all keys are present for translations.
pub fn verify_resources(ls: &LanguageStrings) -> bool {
	for key in TranslatedKey::iter() {
		let mut is_available = false;
		for strs in &ls.kvs {
			if strs.key == key {
				is_available = true;
			}
		}
		if !is_available {
			return false;
		}
	}
	true
}

macro_rules! add_language {
	($func_name:ident, $file_name:expr) => {
		pub fn $func_name() -> LanguageStrings {
			let strings = serde_xml_rs::from_str(include_str!($file_name)).unwrap();
			if !verify_resources(&strings) {
				panic!("The language XML for {} is not correct.", $file_name);
			}
			strings
		}
	}
}

add_language!(en_lang, "../translations/en.xml");
add_language!(fr_lang, "../translations/fr.xml");
