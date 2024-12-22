#![feature(test)]
pub struct Fixture<T> {
    content: T,
    path: &'static str,
}

impl<T> Fixture<T> {
    #[doc(hidden)]
    /// Creates a new fixture from the given content and path.
    pub fn new(content: T, path: &'static str) -> Self {
        Self { content, path }
    }

    /// Returns a reference to the content of the fixture.
    pub fn content(&self) -> &T {
        &self.content
    }

    /// Consumes the fixture and returns the content.
    pub fn into_content(self) -> T {
        self.content
    }

    /// Returns the absolute path of the fixture.
    pub const fn path(&self) -> &'static str {
        self.path
    }
}

pub use bench_test_macros::*;
