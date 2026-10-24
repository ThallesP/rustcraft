//! com/mojang/math/Divisor.java
//!
//! A whole number split into `denominator` whole parts that differ by at most one, the remainder
//! spread over them as it builds up: what a grid layout gives each row or column a child spans.
//! fastutil's `IntIterator` is Rust's `Iterator<Item = i32>`, which ends where Java's
//! `nextInt` throws `NoSuchElementException`.

pub struct Divisor {
    denominator: i32,
    quotient: i32,
    r#mod: i32,
    returned_parts: i32,
    remainder: i32,
}

impl Divisor {
    pub fn new(numerator: i32, denominator: i32) -> Divisor {
        let (quotient, r#mod) = if denominator > 0 {
            (numerator / denominator, numerator % denominator)
        } else {
            (0, 0)
        };
        Divisor {
            denominator,
            quotient,
            r#mod,
            returned_parts: 0,
            remainder: 0,
        }
    }

    pub fn has_next(&self) -> bool {
        self.returned_parts < self.denominator
    }

    /// `nextInt`, which panics where Java throws `NoSuchElementException`.
    pub fn next_int(&mut self) -> i32 {
        if !self.has_next() {
            panic!("java.util.NoSuchElementException");
        }
        let mut next = self.quotient;
        self.remainder = self.remainder.wrapping_add(self.r#mod);
        if self.remainder >= self.denominator {
            self.remainder -= self.denominator;
            next += 1;
        }
        self.returned_parts += 1;
        next
    }
}

impl Iterator for Divisor {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        self.has_next().then(|| self.next_int())
    }
}
