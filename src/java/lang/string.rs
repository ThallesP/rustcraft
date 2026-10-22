//! java.lang.String, where it differs from Rust's str.

pub trait JavaString {
    /// Runs over UTF-16 code units, so anything outside ASCII hashes differently than over bytes or chars.
    fn hash_code(&self) -> i32;

    /// The difference of the first UTF-16 code units that differ, or else of the lengths in
    /// them: not just the sign, and not in code point order beyond the BMP.
    fn compare_to(&self, another_string: &str) -> i32;
}

impl JavaString for str {
    fn hash_code(&self) -> i32 {
        self.encode_utf16().fold(0, |hash: i32, unit| {
            hash.wrapping_mul(31).wrapping_add(i32::from(unit))
        })
    }

    fn compare_to(&self, another_string: &str) -> i32 {
        let mismatch = self
            .encode_utf16()
            .zip(another_string.encode_utf16())
            .find(|(c1, c2)| c1 != c2);
        match mismatch {
            Some((c1, c2)) => i32::from(c1) - i32::from(c2),
            None => {
                self.encode_utf16().count() as i32 - another_string.encode_utf16().count() as i32
            }
        }
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
        golden.check(
            "compareTo(Ljava/lang/String;)I",
            |string: String, another_string: String| string.compare_to(&another_string),
        );
        golden.finish();
    }
}
