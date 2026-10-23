//! net/minecraft/util/ProblemReporter.java
//!
//! Where reading a block entity's data reports what it couldn't read. Java's problems are
//! records with a path of where they happened (`PathElement`, `forChild`), which a
//! `ScopedCollector` logs at once when it's closed; here a problem is its description, and the
//! collector logs each to standard error as it's dropped, with its path's name. Not ported: the
//! paths of children and `Collector`'s other uses (the server's).

use std::cell::RefCell;

pub trait ProblemReporter {
    fn report(&self, problem: String);
}

/// `ProblemReporter.DISCARDING`.
pub struct Discarding;

impl ProblemReporter for Discarding {
    fn report(&self, _problem: String) {}
}

/// `ProblemReporter.DISCARDING`.
pub const DISCARDING: Discarding = Discarding;

/// `ProblemReporter.ScopedCollector`: the problems, logged when it goes out of scope.
pub struct ScopedCollector {
    path: String,
    problems: RefCell<Vec<String>>,
}

impl ScopedCollector {
    pub fn new(path: String) -> ScopedCollector {
        ScopedCollector {
            path,
            problems: RefCell::new(Vec::new()),
        }
    }
}

impl ProblemReporter for ScopedCollector {
    fn report(&self, problem: String) {
        self.problems.borrow_mut().push(problem);
    }
}

impl Drop for ScopedCollector {
    fn drop(&mut self) {
        let problems = self.problems.borrow();
        if !problems.is_empty() {
            eprintln!("[{}]: {}", self.path, problems.join("\n\t"));
        }
    }
}
