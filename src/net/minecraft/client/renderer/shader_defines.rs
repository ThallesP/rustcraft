//! net/minecraft/client/renderer/ShaderDefines.java
//!
//! `values` is Guava's `ImmutableMap` and `flags` its `ImmutableSet`, both in the order they were
//! put, so they're a `Vec` of pairs and a `Vec` without repeats. Not ported yet: `CODEC`
//! (DataFixerUpper), which post chains' JSON reads defines with.
//!
//! `define(String, float)` writes `String.valueOf(float)`, Java's `Float.toString`, which isn't
//! ported. For a float from 0.001 up to 10^7 it's the shortest decimal that reads back as the
//! float, with at least one digit after the point, which is Rust's `{:?}` there (the game
//! defines 0.1 and 0.5, which RenderPipelines' golden checks). Outside that range Java writes an
//! exponent (`1.0E-4`), so the port panics there.

/// `#define`s a pipeline compiles its shaders with: valued ones, then flags.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShaderDefines {
    values: Vec<(String, String)>,
    flags: Vec<String>,
}

impl ShaderDefines {
    pub const EMPTY: ShaderDefines = ShaderDefines {
        values: Vec::new(),
        flags: Vec::new(),
    };

    pub fn new(values: Vec<(String, String)>, flags: Vec<String>) -> ShaderDefines {
        ShaderDefines { values, flags }
    }

    pub fn builder() -> Builder {
        Builder::default()
    }

    pub fn values(&self) -> &[(String, String)] {
        &self.values
    }

    pub fn flags(&self) -> &[String] {
        &self.flags
    }

    /// `ImmutableMap.Builder.buildKeepingLast()`: an overridden key keeps its first place and
    /// takes the later value.
    pub fn with_overrides(&self, defines: &ShaderDefines) -> ShaderDefines {
        if self.is_empty() {
            return defines.clone();
        }
        if defines.is_empty() {
            return self.clone();
        }

        let mut values = self.values.clone();
        for (key, value) in &defines.values {
            match values.iter_mut().find(|(existing, _)| existing == key) {
                Some(entry) => entry.1 = value.clone(),
                None => values.push((key.clone(), value.clone())),
            }
        }
        let mut flags = self.flags.clone();
        for flag in &defines.flags {
            if !flags.contains(flag) {
                flags.push(flag.clone());
            }
        }
        ShaderDefines { values, flags }
    }

    pub fn as_source_directives(&self) -> String {
        let mut directives = String::new();
        for (key, value) in &self.values {
            directives.push_str(&format!("#define {key} {value}\n"));
        }
        for flag in &self.flags {
            directives.push_str(&format!("#define {flag}\n"));
        }
        directives
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty() && self.flags.is_empty()
    }
}

/// `ShaderDefines.Builder`.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    values: Vec<(String, String)>,
    flags: Vec<String>,
}

impl Builder {
    /// Panics on a blank value, where Java throws `IllegalArgumentException`.
    pub fn define(&mut self, key: &str, value: &str) -> &mut Builder {
        assert!(!value.trim().is_empty(), "Cannot define empty string");
        self.values
            .push((key.to_owned(), Builder::escape_new_lines(value)));
        self
    }

    fn escape_new_lines(value: &str) -> String {
        value.replace('\n', "\\\n")
    }

    /// `define(String, float)`, which writes `String.valueOf(float)` (see the file's header).
    pub fn define_f32(&mut self, key: &str, value: f32) -> &mut Builder {
        assert!(
            value == 0.0 || (1.0e-3..1.0e7).contains(&value.abs()),
            "Float.toString's exponent form isn't ported ({value:?})"
        );
        self.values.push((key.to_owned(), format!("{value:?}")));
        self
    }

    /// `define(String, int)`.
    pub fn define_i32(&mut self, key: &str, value: i32) -> &mut Builder {
        self.values.push((key.to_owned(), value.to_string()));
        self
    }

    /// `define(String)`, a flag.
    pub fn define_flag(&mut self, key: &str) -> &mut Builder {
        if !self.flags.iter().any(|flag| flag == key) {
            self.flags.push(key.to_owned());
        }
        self
    }

    /// Panics on a key defined twice, where Guava's `ImmutableMap.Builder.build()` throws
    /// `IllegalArgumentException`.
    pub fn build(&self) -> ShaderDefines {
        for (index, (key, _)) in self.values.iter().enumerate() {
            assert!(
                self.values[..index]
                    .iter()
                    .all(|(earlier, _)| earlier != key),
                "Multiple entries with same key: {key}"
            );
        }
        ShaderDefines {
            values: self.values.clone(),
            flags: self.flags.clone(),
        }
    }
}
