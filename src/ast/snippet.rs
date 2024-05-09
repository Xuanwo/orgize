use crate::syntax::SyntaxKind;

use super::{filter_token, Snippet, Token};

impl Snippet {
    /// ```rust
    /// use orgize::{Org, ast::Snippet};
    ///
    /// let snippet = Org::parse("@@BACKEND:VALUE@@").first_node::<Snippet>().unwrap();
    /// assert_eq!(snippet.backend(), "BACKEND");
    /// ```
    pub fn backend(&self) -> Token {
        self.syntax
            .children_with_tokens()
            .find_map(filter_token(SyntaxKind::TEXT))
            .expect("snippet must contains TEXT")
    }

    /// ```rust
    /// use orgize::{Org, ast::Snippet};
    ///
    /// let snippet = Org::parse("@@BACKEND:@@").first_node::<Snippet>().unwrap();
    /// assert_eq!(snippet.value(), "");
    /// let snippet = Org::parse("@@BACKEND:VALUE@@").first_node::<Snippet>().unwrap();
    /// assert_eq!(snippet.value(), "VALUE");
    /// ```
    pub fn value(&self) -> Token {
        self.syntax
            .children_with_tokens()
            .filter_map(filter_token(SyntaxKind::TEXT))
            .nth(1)
            .expect("snippet must contains two TEXT")
    }
}
