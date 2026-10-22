//! net/minecraft/util/FileUtil.java
//!
//! Only the resource paths' helpers so far: `decomposePath`, `resolvePath`, `isValidPathSegment`
//! and `validatePath`. `decomposePath`'s `DataResult` error is the `Err`, with Java's message.

use std::path::{Path, PathBuf};

pub struct FileUtil;

impl FileUtil {
    /// The path's segments between slashes, each non-empty, not `.` or `..`, and only of
    /// `[-._a-z0-9]`.
    pub fn decompose_path(path: &str) -> Result<Vec<String>, String> {
        if !path.contains('/') {
            return match path {
                "" | "." | ".." => Err(format!("Invalid path '{path}'")),
                _ if !FileUtil::contains_allowed_characters_only(path) => {
                    Err(format!("Invalid path '{path}'"))
                }
                _ => Ok(vec![path.to_owned()]),
            };
        }
        let mut result = Vec::new();
        for segment in path.split('/') {
            if matches!(segment, "" | "." | "..")
                || !FileUtil::contains_allowed_characters_only(segment)
            {
                return Err(format!("Invalid segment '{segment}' in path '{path}'"));
            }
            result.push(segment.to_owned());
        }
        Ok(result)
    }

    pub fn resolve_path(root: &Path, segments: &[String]) -> PathBuf {
        let mut path = root.to_path_buf();
        for segment in segments {
            path.push(segment);
        }
        path
    }

    fn contains_allowed_characters_only(segment: &str) -> bool {
        !segment.is_empty()
            && segment
                .bytes()
                .all(|c| matches!(c, b'-' | b'.' | b'_' | b'a'..=b'z' | b'0'..=b'9'))
    }

    pub fn is_valid_path_segment(segment: &str) -> bool {
        !matches!(segment, "." | "..") && FileUtil::contains_allowed_characters_only(segment)
    }

    /// Panics where Java throws `IllegalArgumentException`.
    pub fn validate_path(path: &[&str]) {
        assert!(!path.is_empty(), "Path must have at least one element");
        for segment in path {
            assert!(
                FileUtil::is_valid_path_segment(segment),
                "Illegal segment {segment} in path {path:?}"
            );
        }
    }
}
