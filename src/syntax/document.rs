use nom::{combinator::opt, IResult};

use super::{
    combinator::{blank_lines, node, GreenElement},
    drawer::property_drawer_node,
    headline::{headline_node, section_node},
    input::Input,
    SyntaxKind::*,
};

#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub fn document_node(input: Input) -> IResult<Input, GreenElement, ()> {
    crate::lossless_parser!(document_node_base, input)
}

fn document_node_base(input: Input) -> IResult<Input, GreenElement, ()> {
    if input.is_empty() {
        return Ok((input, node(DOCUMENT, [])));
    }

    let mut children = vec![];

    let (input, property_drawer) = opt(property_drawer_node)(input)?;
    if let Some(property_drawer) = property_drawer {
        children.push(property_drawer);
    }

    let (input, pre_blank) = blank_lines(input)?;

    children.extend(pre_blank);

    if input.is_empty() {
        return Ok((input, node(DOCUMENT, children)));
    }

    let (input, section) = opt(section_node)(input)?;
    if let Some(section) = section {
        children.push(section);
    }

    let mut i = input;
    while !i.is_empty() {
        let (input, headline) = headline_node(i)?;
        debug_assert!(i.len() > input.len(), "{} > {}", i.len(), input.len(),);
        i = input;
        children.push(headline);
    }

    Ok((i, node(DOCUMENT, children)))
}

#[test]
fn parse() {
    use crate::ast::Document;
    use crate::tests::to_ast;

    let to_document = to_ast::<Document>(document_node);

    insta::assert_debug_snapshot!(
        to_document("").syntax,
        @r###"
    DOCUMENT@0..0
    "###
    );

    insta::assert_debug_snapshot!(
        to_document("\n  \n\n").syntax,
        @r###"
    DOCUMENT@0..5
      BLANK_LINE@0..1 "\n"
      BLANK_LINE@1..4 "  \n"
      BLANK_LINE@4..5 "\n"
    "###
    );

    insta::assert_debug_snapshot!(
        to_document("section").syntax,
        @r###"
    DOCUMENT@0..7
      SECTION@0..7
        PARAGRAPH@0..7
          TEXT@0..7 "section"
    "###
    );

    insta::assert_debug_snapshot!(
        to_document("\n* section").syntax,
        @r###"
    DOCUMENT@0..10
      BLANK_LINE@0..1 "\n"
      HEADLINE@1..10
        HEADLINE_STARS@1..2 "*"
        WHITESPACE@2..3 " "
        HEADLINE_TITLE@3..10
          TEXT@3..10 "section"
    "###
    );

    insta::assert_debug_snapshot!(
        to_document("\n** heading 2\n* heading 1").syntax,
        @r###"
    DOCUMENT@0..25
      BLANK_LINE@0..1 "\n"
      HEADLINE@1..14
        HEADLINE_STARS@1..3 "**"
        WHITESPACE@3..4 " "
        HEADLINE_TITLE@4..13
          TEXT@4..13 "heading 2"
        NEW_LINE@13..14 "\n"
      HEADLINE@14..25
        HEADLINE_STARS@14..15 "*"
        WHITESPACE@15..16 " "
        HEADLINE_TITLE@16..25
          TEXT@16..25 "heading 1"
    "###
    );

    insta::assert_debug_snapshot!(
        to_document("section\n** heading 2\n*heading 1").syntax,
        @r###"
    DOCUMENT@0..31
      SECTION@0..8
        PARAGRAPH@0..8
          TEXT@0..8 "section\n"
      HEADLINE@8..31
        HEADLINE_STARS@8..10 "**"
        WHITESPACE@10..11 " "
        HEADLINE_TITLE@11..20
          TEXT@11..20 "heading 2"
        NEW_LINE@20..21 "\n"
        SECTION@21..31
          PARAGRAPH@21..31
            TEXT@21..31 "*heading 1"
    "###
    );
}
