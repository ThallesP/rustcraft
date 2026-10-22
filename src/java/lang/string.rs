//! java.lang.String, where it differs from Rust's str.

pub trait JavaString {
    /// Runs over UTF-16 code units, so anything outside ASCII hashes differently than over bytes or chars.
    fn hash_code(&self) -> i32;
}

impl JavaString for str {
    fn hash_code(&self) -> i32 {
        self.encode_utf16().fold(0, |hash: i32, unit| {
            hash.wrapping_mul(31).wrapping_add(i32::from(unit))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::JavaString;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("java.lang.String");
        golden.check("hashCode()I", |string: String| string.hash_code());
        golden.finish();
    }
}
