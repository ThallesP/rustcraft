//! net/minecraft/locale/Language.java, with DeprecatedTranslationsInfo.java
//!
//! The translations a translatable text is looked up in. Java's default instance reads
//! `/assets/minecraft/lang/en_us.json` from the client jar, then applies `deprecated.json`'s
//! removals and renames; the client injects a `ClientLanguage` loaded from the resource packs,
//! which for the vanilla pack in `en_us` is the same file, so the port reads it from
//! `vanilla/<version>/resources/` once, on first use. Both files are read with Gson's strict
//! parser (`StrictJsonParser`, which `DeprecatedTranslationsInfo` uses; `en_us.json` is valid JSON
//! however Gson reads it). `inject`, `getVisualOrder` (bidirectional text) and right-to-left come
//! with the client's languages.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::json_parser::JsonParser;
use crate::net::minecraft::server::packs::vanilla_pack_resources::VanillaPackResources;

pub struct Language {
    storage: HashMap<String, String>,
}

static INSTANCE: LazyLock<Language> = LazyLock::new(Language::load_default);

impl Language {
    pub const DEFAULT: &'static str = "en_us";

    pub fn get_instance() -> &'static Language {
        &INSTANCE
    }

    // `loadDefault`: en_us, then the deprecated keys removed and renamed.
    fn load_default() -> Language {
        let lang = VanillaPackResources::vanilla_jar_root().join("assets/minecraft/lang");
        let mut storage = HashMap::new();
        match std::fs::read_to_string(lang.join("en_us.json")) {
            Ok(text) => Language::load_from_json(&text, &mut |key, value| {
                storage.insert(key, value);
            }),
            Err(error) => eprintln!("Couldn't read strings from en_us.json: {error}"),
        }
        if let Ok(text) = std::fs::read_to_string(lang.join("deprecated.json")) {
            apply_deprecated(&text, &mut storage);
        }
        Language { storage }
    }

    /// `loadFromJson`: each entry's string (a primitive's text), with an unsupported `%d` or
    /// `%f` (and their widths and precisions) made `%s`.
    pub fn load_from_json(text: &str, output: &mut dyn FnMut(String, String)) {
        let Ok(JsonElement::Object(entries)) = JsonParser::parse_strict(text) else {
            eprintln!("Couldn't read strings: not a JSON object");
            return;
        };
        for (key, value) in entries.entry_set() {
            if !value.is_json_primitive() {
                eprintln!("Expected {key} to be a string");
                return;
            }
            output(
                key.clone(),
                replace_unsupported_formats(&value.get_as_string()),
            );
        }
    }

    /// `getOrDefault(String)`: the translation, or the key itself.
    pub fn get_or_default(&self, element_id: &str) -> String {
        self.get_or_default_with(element_id, element_id)
    }

    /// `getOrDefault(String, String)`.
    pub fn get_or_default_with(&self, element_id: &str, default_value: &str) -> String {
        self.storage
            .get(element_id)
            .cloned()
            .unwrap_or_else(|| default_value.to_owned())
    }

    pub fn has(&self, element_id: &str) -> bool {
        self.storage.contains_key(element_id)
    }

    pub fn is_default_right_to_left(&self) -> bool {
        false
    }
}

// `DeprecatedTranslationsInfo.applyToMap`: the removed keys taken out, then each renamed key's
// translation moved to its new key (or the new key taken out when the old one's missing), in the
// file's order.
fn apply_deprecated(text: &str, translations: &mut HashMap<String, String>) {
    let Ok(JsonElement::Object(info)) = JsonParser::parse_strict(text) else {
        return;
    };
    if let Some(JsonElement::Array(removed)) = info.get("removed") {
        for key in removed {
            translations.remove(&key.get_as_string());
        }
    }
    if let Some(JsonElement::Object(renamed)) = info.get("renamed") {
        for (from_key, to_key) in renamed.entry_set() {
            let to_key = to_key.get_as_string();
            match translations.remove(from_key) {
                Some(value) => {
                    translations.insert(to_key, value);
                }
                None => {
                    translations.remove(&to_key);
                }
            }
        }
    }
}

// `UNSUPPORTED_FORMAT_PATTERN.matcher(text).replaceAll("%$1s")`, the pattern
// `%(\d+\$)?[\d.]*[df]`.
fn replace_unsupported_formats(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut copied = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'%' {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        let digits = bytes[j..].iter().take_while(|b| b.is_ascii_digit()).count();
        let group = if digits > 0 && bytes.get(j + digits) == Some(&b'$') {
            let group = &text[j..j + digits + 1];
            j += digits + 1;
            group
        } else {
            ""
        };
        j += bytes[j..]
            .iter()
            .take_while(|b| b.is_ascii_digit() || **b == b'.')
            .count();
        if matches!(bytes.get(j), Some(b'd' | b'f')) {
            result.push_str(&text[copied..i]);
            result.push('%');
            result.push_str(group);
            result.push('s');
            copied = j + 1;
            i = j + 1;
        } else {
            i += 1;
        }
    }
    result.push_str(&text[copied..]);
    result
}
