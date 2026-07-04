/// Parses the WikiText as defined [here](https://tiddlywiki.com/static/WikiText.html)
use winnow::{
    ModalResult, Parser,
    combinator::{alt, eof, fail, not, repeat, repeat_till},
    stream::AsChar,
    token::{literal, take_till, take_until, take_while},
};

use crate::abstract_syntax::Inline;

const MARKERS: &[&str] = &["''", "//", "__", "^^", "~~", "`", "@@"];

fn italics(input: &mut &str) -> ModalResult<Inline> {
    let text = (literal("//"), take_until(0.., "//"), literal("//"))
        .parse_next(input)?
        .1;
    Ok(Inline::Italics {
        text: text.to_string(),
    })
}

fn bold(input: &mut &str) -> ModalResult<Inline> {
    let text = (literal("''"), take_until(0.., "''"), literal("''"))
        .parse_next(input)?
        .1;

    Ok(Inline::Bold {
        text: text.to_string(),
    })
}

fn underlined(input: &mut &str) -> ModalResult<Inline> {
    let text = (literal("__"), take_until(0.., "__"), literal("__"))
        .parse_next(input)?
        .1;

    Ok(Inline::Underlined {
        text: text.to_string(),
    })
}

/// Consume plain text up until the next formatting marker, or to EOF.
fn plain_text(input: &mut &str) -> ModalResult<Inline> {
    let stop = MARKERS.iter().filter_map(|m| input.find(m)).min();

    match stop {
        Some(0) => fail.parse_next(input),
        Some(pos) => {
            let (consumed, rest) = input.split_at(pos);
            *input = rest;
            Ok(Inline::PlainText {
                text: consumed.to_string(),
            })
        }
        None => {
            let consumed = *input;
            *input = "";
            Ok(Inline::PlainText {
                text: consumed.to_string(),
            })
        }
    }
}

fn inline(input: &mut &str) -> ModalResult<Inline> {
    alt((italics, bold, underlined, plain_text)).parse_next(input)
}

pub fn parse_wiki_text(input: &mut &str) -> ModalResult<Vec<Inline>> {
    // repeat(0.., inline).parse_next(input)
    let r: ModalResult<(Vec<Inline>, _)> = repeat_till(0.., inline, eof).parse_next(input);

    match r {
        Ok(v) => Ok(v.0),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_italics() {
        let mut text = "This is some text that //has italics// in the middle.";
        let r = parse_wiki_text(&mut text);

        assert!(r.is_ok());

        let v = r.unwrap();
        assert_eq!(v.len(), 3);

        let expected: Vec<Inline> = vec![
            Inline::plaintext("This is some text that "),
            Inline::italics("has italics"),
            Inline::plaintext(" in the middle."),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_bold() {
        let mut text = "Text that has ''bold text'' in it.";
        let r = parse_wiki_text(&mut text);

        assert!(r.is_ok());

        let v = r.unwrap();
        assert_eq!(v.len(), 3);

        let expected: Vec<Inline> = vec![
            Inline::PlainText {
                text: "Text that has ".to_string(),
            },
            Inline::Bold {
                text: "bold text".to_string(),
            },
            Inline::PlainText {
                text: " in it.".to_string(),
            },
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_bold_and_italics() {
        let mut text =
            "Some text with a mixture of //italicised text// and also ''some bold text'' in it.";
        let r = parse_wiki_text(&mut text);

        assert!(r.is_ok());

        let v = r.unwrap();
        assert_eq!(v.len(), 5);

        let expected: Vec<Inline> = vec![
            Inline::PlainText {
                text: "Some text with a mixture of ".to_string(),
            },
            Inline::Italics {
                text: "italicised text".to_string(),
            },
            Inline::PlainText {
                text: " and also ".to_string(),
            },
            Inline::Bold {
                text: "some bold text".to_string(),
            },
            Inline::PlainText {
                text: " in it.".to_string(),
            },
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_underlined() {
        let mut text = "This is __some underlined text__.";
        let r = parse_wiki_text(&mut text);

        assert!(r.is_ok());

        let v = r.unwrap();
        assert_eq!(v.len(), 3);

        let expected: Vec<Inline> = vec![
            Inline::plaintext("This is "),
            Inline::underlined("some underlined text"),
            Inline::plaintext("."),
        ];

        assert_eq!(v, expected);
    }
}
