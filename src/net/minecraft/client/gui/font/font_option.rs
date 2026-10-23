//! net/minecraft/client/gui/font/FontOption.java
//!
//! The font options a provider can be filtered by: the Unicode font forced, and Japanese glyph
//! variants. A `Filter` is Java's map of options to the value each must have; the order of its
//! entries doesn't change what it answers.

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontOption {
    UNIFORM,
    JAPANESE_VARIANTS,
}

impl FontOption {
    pub fn get_serialized_name(self) -> &'static str {
        match self {
            FontOption::UNIFORM => "uniform",
            FontOption::JAPANESE_VARIANTS => "jp",
        }
    }

    fn by_name(name: &str) -> Option<FontOption> {
        [FontOption::UNIFORM, FontOption::JAPANESE_VARIANTS]
            .into_iter()
            .find(|option| option.get_serialized_name() == name)
    }
}

/// `FontOption.Filter`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filter {
    values: Vec<(FontOption, bool)>,
}

impl Filter {
    pub const ALWAYS_PASS: Filter = Filter { values: Vec::new() };

    pub fn new(values: Vec<(FontOption, bool)>) -> Filter {
        Filter { values }
    }

    pub fn apply(&self, options: &[FontOption]) -> bool {
        self.values
            .iter()
            .all(|(option, value)| options.contains(option) == *value)
    }

    /// `merge`: the other's values, with this one's over them.
    pub fn merge(&self, other: &Filter) -> Filter {
        let mut options = other.values.clone();
        for (option, value) in &self.values {
            match options.iter_mut().find(|(existing, _)| existing == option) {
                Some(entry) => entry.1 = *value,
                None => options.push((*option, *value)),
            }
        }
        Filter { values: options }
    }

    /// `CODEC`: a map of options' names to booleans.
    pub fn decode(value: &serde_json::Value) -> Result<Filter, String> {
        let object = value
            .as_object()
            .ok_or_else(|| format!("Not a map: {value}"))?;
        let mut values = Vec::new();
        for (name, value) in object {
            let option =
                FontOption::by_name(name).ok_or_else(|| format!("Unknown element name:{name}"))?;
            let value = value
                .as_bool()
                .ok_or_else(|| format!("Not a boolean: {value}"))?;
            values.push((option, value));
        }
        Ok(Filter { values })
    }
}
