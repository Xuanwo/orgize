use rowan::{ast::AstNode, NodeOrToken};

use crate::{syntax::SyntaxKind, SyntaxElement};

use super::{filter_token, Clock, Drawer, Headline, Section, Timestamp, Token};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TodoType {
    Todo,
    Done,
}

impl Headline {
    /// Return level of this headline
    ///
    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let hdl = Org::parse("* ").first_node::<Headline>().unwrap();
    /// assert_eq!(hdl.level(), 1);
    /// let hdl = Org::parse("****** hello").first_node::<Headline>().unwrap();
    /// assert_eq!(hdl.level(), 6);
    /// ```
    pub fn level(&self) -> usize {
        self.syntax
            .children_with_tokens()
            .find_map(filter_token(SyntaxKind::HEADLINE_STARS))
            .map_or_else(
                || {
                    debug_assert!(false, "headline must contains HEADLINE_STARS");
                    0
                },
                |stars| stars.len(),
            )
    }

    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let hdl = Org::parse("* TODO a").first_node::<Headline>().unwrap();
    /// assert_eq!(hdl.todo_keyword().unwrap(), "TODO");
    /// ```
    pub fn todo_keyword(&self) -> Option<Token> {
        self.syntax
            .children_with_tokens()
            .find_map(|elem| match elem {
                NodeOrToken::Token(tk)
                    if tk.kind() == SyntaxKind::HEADLINE_KEYWORD_TODO
                        || tk.kind() == SyntaxKind::HEADLINE_KEYWORD_DONE =>
                {
                    Some(Token(tk))
                }
                _ => None,
            })
    }

    /// ```rust
    /// use orgize::{Org, ast::{Headline, TodoType}};
    ///
    /// let hdl = Org::parse("* TODO a").first_node::<Headline>().unwrap();
    /// assert_eq!(hdl.todo_type().unwrap(), TodoType::Todo);
    /// let hdl = Org::parse("*** DONE a").first_node::<Headline>().unwrap();
    /// assert_eq!(hdl.todo_type().unwrap(), TodoType::Done);
    /// ```
    pub fn todo_type(&self) -> Option<TodoType> {
        self.syntax
            .children_with_tokens()
            .find_map(|elem| match elem {
                NodeOrToken::Token(tk) if tk.kind() == SyntaxKind::HEADLINE_KEYWORD_TODO => {
                    Some(TodoType::Todo)
                }
                NodeOrToken::Token(tk) if tk.kind() == SyntaxKind::HEADLINE_KEYWORD_DONE => {
                    Some(TodoType::Done)
                }
                _ => None,
            })
    }

    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let hdl = Org::parse("* TODO a").first_node::<Headline>().unwrap();
    /// assert!(hdl.is_todo());
    /// let hdl = Org::parse("* a").first_node::<Headline>().unwrap();
    /// assert!(!hdl.is_todo());
    /// ```
    pub fn is_todo(&self) -> bool {
        matches!(self.todo_type(), Some(TodoType::Todo))
    }

    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let hdl = Org::parse("* DONE a").first_node::<Headline>().unwrap();
    /// assert!(hdl.is_done());
    /// let hdl = Org::parse("* a").first_node::<Headline>().unwrap();
    /// assert!(!hdl.is_done());
    /// ```
    pub fn is_done(&self) -> bool {
        matches!(self.todo_type(), Some(TodoType::Done))
    }

    /// Returns parsed title
    ///
    /// ```rust
    /// use orgize::{Org, ast::Headline, SyntaxKind};
    ///
    /// let hdl = Org::parse("*** abc *abc* /abc/ :tag:").first_node::<Headline>().unwrap();
    /// let title = hdl.title().collect::<Vec<_>>();
    /// assert_eq!(title[1].kind(), SyntaxKind::BOLD);
    /// assert_eq!(title[1].to_string(), "*abc*");
    /// assert_eq!(title[3].kind(), SyntaxKind::ITALIC);
    /// assert_eq!(title[3].to_string(), "/abc/");
    /// ```
    pub fn title(&self) -> impl Iterator<Item = SyntaxElement> {
        self.syntax
            .children()
            .find(|n| n.kind() == SyntaxKind::HEADLINE_TITLE)
            .into_iter()
            .flat_map(|n| n.children_with_tokens())
    }

    /// Returns title raw string
    ///
    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let hdl = Org::parse("*** abc *abc* /abc/ :tag:").first_node::<Headline>().unwrap();
    /// let title = hdl.title_raw();
    /// assert_eq!(title, "abc *abc* /abc/ ");
    /// ```
    pub fn title_raw(&self) -> String {
        self.syntax
            .children()
            .find(|n| n.kind() == SyntaxKind::HEADLINE_TITLE)
            .map(|n| n.to_string())
            .unwrap_or_default()
    }

    /// Return `true` if this headline contains a COMMENT keyword
    ///      
    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let hdl = Org::parse("* COMMENT").first_node::<Headline>().unwrap();
    /// assert!(hdl.is_commented());
    /// let hdl = Org::parse("* COMMENT hello").first_node::<Headline>().unwrap();
    /// assert!(hdl.is_commented());
    /// let hdl = Org::parse("* hello").first_node::<Headline>().unwrap();
    /// assert!(!hdl.is_commented());
    /// ```
    pub fn is_commented(&self) -> bool {
        self.title()
            .next()
            .map(|first| {
                if let Some(t) = first.as_token() {
                    let text = t.text();
                    t.kind() == SyntaxKind::TEXT
                        && text.starts_with("COMMENT")
                        && (text.len() == 7 || text[7..].starts_with(char::is_whitespace))
                } else {
                    false
                }
            })
            .unwrap_or_default()
    }

    /// Return `true` if this headline contains an archive tag
    ///
    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let hdl = Org::parse("* hello :ARCHIVE:").first_node::<Headline>().unwrap();
    /// assert!(hdl.is_archived());
    /// let hdl = Org::parse("* hello :ARCHIVED:").first_node::<Headline>().unwrap();
    /// assert!(!hdl.is_archived());
    /// ```
    pub fn is_archived(&self) -> bool {
        self.tags().any(|t| t == "ARCHIVE")
    }

    /// Returns this headline's closed timestamp, or `None` if not set.
    pub fn closed(&self) -> Option<Timestamp> {
        self.planning().and_then(|planning| planning.closed())
    }

    /// Returns this headline's scheduled timestamp, or `None` if not set.
    pub fn scheduled(&self) -> Option<Timestamp> {
        self.planning().and_then(|planning| planning.scheduled())
    }

    /// Returns this headline's deadline timestamp, or `None` if not set.
    pub fn deadline(&self) -> Option<Timestamp> {
        self.planning().and_then(|planning| planning.deadline())
    }

    /// Returns an iterator of text token in this tags
    ///
    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let tags_vec = |input: &str| {
    ///     let hdl = Org::parse(input).first_node::<Headline>().unwrap();
    ///     let tags: Vec<_> = hdl.tags().map(|t| t.to_string()).collect();
    ///     tags
    /// };
    ///
    /// assert_eq!(tags_vec("* :tag:"), vec!["tag".to_string()]);
    /// assert_eq!(tags_vec("* [#A] :::::a2%:"), vec!["a2%".to_string()]);
    /// assert_eq!(tags_vec("* TODO :tag:  :a2%:"), vec!["tag".to_string(), "a2%".to_string()]);
    /// assert_eq!(tags_vec("* title :tag:a2%:"), vec!["tag".to_string(), "a2%".to_string()]);
    /// ```
    pub fn tags(&self) -> impl Iterator<Item = Token> {
        self.syntax
            .children()
            .find(|n| n.kind() == SyntaxKind::HEADLINE_TAGS)
            .into_iter()
            .flat_map(|t| t.children_with_tokens())
            .filter_map(filter_token(SyntaxKind::TEXT))
    }

    /// Returns priority text
    ///
    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let hdl = Org::parse("* [#A]").first_node::<Headline>().unwrap();
    /// assert_eq!(hdl.priority().unwrap(), "A");
    /// let hdl = Org::parse("** DONE [#B]::").first_node::<Headline>().unwrap();
    /// assert_eq!(hdl.priority().unwrap(), "B");
    /// let hdl = Org::parse("* [#破]").first_node::<Headline>().unwrap();
    /// assert_eq!(hdl.priority().unwrap(), "破");
    /// ```
    pub fn priority(&self) -> Option<Token> {
        self.syntax
            .children()
            .find(|n| n.kind() == SyntaxKind::HEADLINE_PRIORITY)
            .and_then(|n| {
                n.children_with_tokens()
                    .find_map(filter_token(SyntaxKind::TEXT))
            })
    }

    /// Returns an iterator of clock element affiliated with this headline
    ///
    /// ```rust
    /// use orgize::{Org, ast::Headline};
    ///
    /// let org = Org::parse(r#"* TODO
    /// foo
    /// :LOGBOOK:
    /// bar
    /// CLOCK:
    /// CLOCK: [2024-10-12]
    /// baz
    /// CLOCK: [2024-10-12]
    /// [2024-10-12]
    /// :END:
    /// foo"#);
    /// let hdl = org.first_node::<Headline>().unwrap();
    /// assert_eq!(hdl.clocks().count(), 2);
    /// ```
    pub fn clocks(&self) -> impl Iterator<Item = Clock> {
        self.syntax
            .children()
            .flat_map(Section::cast)
            .flat_map(|x| x.syntax.children().filter_map(Drawer::cast))
            .filter(|d| d.name().eq_ignore_ascii_case("LOGBOOK"))
            .filter_map(|d| {
                d.syntax
                    .children()
                    .find(|children| children.kind() == SyntaxKind::DRAWER_CONTENT)
            })
            .flat_map(|x| x.children().filter_map(Clock::cast))
    }
}
