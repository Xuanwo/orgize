use rowan::ast::AstNode;

use crate::Org;

use super::{Document, Keyword, PropertyDrawer};

impl Document {
    /// Returns an iterator of keywords in zeroth section
    ///
    /// ```rust
    /// use orgize::{Org, ast::Document};
    ///
    /// let org = Org::parse(r#"
    /// #+TITLE: hello
    /// #+TITLE: world
    /// #+DATE: today
    /// #+AUTHOR: poi
    /// * headline
    /// #+SOMETHING:"#);
    /// let doc = org.first_node::<Document>().unwrap();
    /// assert_eq!(doc.keywords().count(), 4);
    /// ```
    pub fn keywords(&self) -> impl Iterator<Item = Keyword> {
        self.section()
            .into_iter()
            .flat_map(|section| section.syntax.children().filter_map(Keyword::cast))
    }

    /// Returns the value in top-level `#+TITLE`
    ///
    /// Multiple `#+TITLE` are joined with spaces.
    ///
    /// Returns `None` if file doesn't contain `#+TITLE`
    ///
    /// ```rust
    /// use orgize::{Org, ast::Document};
    ///
    /// let org = Org::parse("#+TITLE: hello\n#+TITLE: world");
    /// let doc = org.first_node::<Document>().unwrap();
    /// assert_eq!(doc.title().unwrap(), "hello world");
    ///
    /// let org = Org::parse("");
    /// let doc = org.first_node::<Document>().unwrap();
    /// assert!(doc.title().is_none());
    /// ```
    pub fn title(&self) -> Option<String> {
        self.keywords()
            .filter(|kw| kw.key().eq_ignore_ascii_case("TITLE"))
            .fold(Option::<String>::None, |acc, cur| {
                let mut s = acc.unwrap_or_default();
                if !s.is_empty() {
                    s.push(' ');
                }
                s.push_str(cur.value().trim());
                Some(s)
            })
    }

    /// Returns top-level properties drawer
    ///
    /// ```rust
    /// use orgize::{Org, ast::Document};
    ///
    /// let org = Org::parse(r#":PROPERTIES:
    /// :ID:       20220718T085035.042592
    /// :END:
    /// #+TITLE: Complete Computing"#);
    ///
    /// let properties = org.document().properties().unwrap();
    /// assert_eq!(properties.to_hash_map().len(), 1);
    /// assert_eq!(properties.get("ID").unwrap(), "20220718T085035.042592");
    /// ```
    pub fn properties(&self) -> Option<PropertyDrawer> {
        rowan::ast::support::child(&self.syntax)
    }
}

impl Org {
    /// Equals to `self.document().title()`, see [Document::title]
    pub fn title(&self) -> Option<String> {
        self.document().title()
    }

    /// Equals to `self.document().keywords()`, see [Document::keywords]
    pub fn keywords(&self) -> impl Iterator<Item = Keyword> {
        self.document().keywords()
    }
}
