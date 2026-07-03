use winnow::{
    ModalResult, Parser,
    combinator::{alt, eof, fail, repeat, repeat_till},
    token::{literal, take_until},
};
/// Parses the WikiText as defined [here](https://tiddlywiki.com/static/WikiText.html)

#[derive(Debug, PartialEq, Clone)]
pub enum Inline {
    PlainText {
        text: String,
    },
    Italics {
        text: String,
    },
    Bold {
        text: String,
    },
    Underlined {
        text: String,
    },
    Superscript {
        text: String,
    },
    Subscript {
        text: String,
    },
    Strikethrough {
        text: String,
    },
    Highlight {
        text: String,
    },
    BlockQuote {
        text: String,
    },
    CodeBlock {
        text: String,
        language: Option<String>,
    },
    Heading {
        text: String,
        level: usize,
    },
    Image {
        width: usize,
        height: usize,
        link: String,
    },
    Link {
        display_text: Option<String>,
        link: String,
    },
    OrderedList {
        level: usize,
    },
    UnorderedList {
        level: usize,
    },
    TableCell {
        text: String,
        heading: bool,
    },
    Transclusion {
        link: String,
    },
}

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

/// Consume plain text up until the next "//" or "''" marker, or to EOF.
fn plain_text(input: &mut &str) -> ModalResult<Inline> {
    // Find the earliest occurrence of either marker.
    let ital_pos = input.find("//");
    let bold_pos = input.find("''");

    let stop = match (ital_pos, bold_pos) {
        (Some(i), Some(b)) => Some(i.min(b)),
        (Some(i), None) => Some(i),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

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
    alt((italics, bold, plain_text)).parse_next(input)
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
            Inline::PlainText {
                text: "This is some text that ".to_string(),
            },
            Inline::Italics {
                text: "has italics".to_string(),
            },
            Inline::PlainText {
                text: " in the middle.".to_string(),
            },
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
}
