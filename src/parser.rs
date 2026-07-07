/// Parses the WikiText as defined [here](https://tiddlywiki.com/static/WikiText.html)
use winnow::{
    ModalResult, Parser,
    ascii::{alphanumeric0, digit1, line_ending, multispace0, space0, space1},
    combinator::{
        alt, delimited, eof, fail, not, opt, preceded, repeat, repeat_till, separated,
        separated_pair, seq, todo,
    },
    stream::AsChar,
    token::{literal, one_of, rest, take, take_till, take_until, take_while},
};

use crate::abstract_syntax::{DimensionField, Dimensions, Inline};

const MARKERS: &[&str] = &[
    "''", "//", "__", "^^", "~~", "`", "@@", "<<<", "```", "!", "[img", "[[", "[ext[",
];

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

fn superscript(input: &mut &str) -> ModalResult<Inline> {
    let marker = "^^";
    let text = (literal(marker), take_until(0.., marker), literal(marker))
        .parse_next(input)?
        .1;

    Ok(Inline::Superscript {
        text: text.to_string(),
    })
}

fn strikethrough(input: &mut &str) -> ModalResult<Inline> {
    let marker = "~~";
    let text = (literal(marker), take_until(0.., marker), literal(marker))
        .parse_next(input)?
        .1;

    Ok(Inline::Strikethrough {
        text: text.to_string(),
    })
}

fn code(input: &mut &str) -> ModalResult<Inline> {
    let marker = "`";
    let text = (literal(marker), take_until(0.., marker), literal(marker))
        .parse_next(input)?
        .1;

    Ok(Inline::Code {
        text: text.to_string(),
    })
}

fn highlight(input: &mut &str) -> ModalResult<Inline> {
    let marker = "@@";
    let text = (literal(marker), take_until(0.., marker), literal(marker))
        .parse_next(input)?
        .1;

    Ok(Inline::Highlight {
        text: text.to_string(),
    })
}

fn blockquote(input: &mut &str) -> ModalResult<Inline> {
    let marker = "<<<";

    let quote = delimited(
        (multispace0, marker, multispace0),
        take_until(0.., marker),
        (multispace0, marker, multispace0),
    )
    .parse_next(input)?
    .trim();

    let citation = take_while(0.., |c| c != '\n' && c != '\r')
        .parse_next(input)?
        .trim();

    let citation_option = (!citation.is_empty()).then(|| citation.to_string());

    Ok(Inline::BlockQuote {
        text: quote.to_string(),
        citation: citation_option,
    })
}

fn codeblock(input: &mut &str) -> ModalResult<Inline> {
    let marker = "```";

    // let r = delimited(marker, alphanumeric0, line_ending);

    let language = preceded(marker, alphanumeric0).parse_next(input)?;
    let code = take_until(0.., marker).parse_next(input)?;
    // Consume the remaining marker.
    literal(marker).parse_next(input)?;

    // Remove any leading new lines
    let code = code.trim_start_matches(['\n', '\r']);

    let language_option = (!language.is_empty()).then(|| language.trim().to_string());

    Ok(Inline::CodeBlock {
        text: code.to_string(),
        language: language_option,
    })
}

fn heading(input: &mut &str) -> ModalResult<Inline> {
    // 1. Count the number of hashes (1 to 6) to determine the heading level
    let hashes: Vec<char> = repeat(1..=6, one_of('!')).parse_next(input)?;
    let level = hashes.len();

    // 2. Consume the required trailing whitespace separating the # and the text
    let _space = space1.parse_next(input)?;

    // 3. Consume everything else on the line as the heading text
    // let text = rest.parse_next(input)?.to_string();
    let text = take_till(0.., |c| c == '\n' || c == '\r').parse_next(input)?;

    Ok(Inline::Heading {
        level,
        text: text.to_string(),
    })
}
/// Parses an image
/// BNF:
/// ```bnf
/// image           = "[img" {whitespace} ["width=" ,number] , ["height=" ,number]  {whitespace} ["height="" ,number] [" , [ caption , "|" ] , image_source , "]]" ;
/// caption         = { any_char_except("|", "]]") } ;
/// image_source    = { any_char_except("]]") } ;
/// ```
fn image(input: &mut &str) -> ModalResult<Inline> {
    let img = seq!(
        "[img",
        space0,
        opt(dimensions),
        space0,
        "[",
        opt((take_until(0.., '|'), take(1usize))),
        (take_until(1.., "]]"), take(2usize))
    )
    .parse_next(input)?;

    let width = img.2.clone().map(|d| d.width).flatten();
    let height = img.2.map(|d| d.height).flatten();
    let caption = img.5.map(|s| String::from(s.0));
    let link = String::from(img.6.0);

    Ok(Inline::Image {
        width,
        height,
        caption,
        link,
    })
}

fn link(input: &mut &str) -> ModalResult<Inline> {
    let link = alt((simple_link, external_link)).parse_next(input)?;

    Ok(link)
}

fn simple_link(input: &mut &str) -> ModalResult<Inline> {
    let link_statement: (&str, Option<(&str, &str)>, (&str, &str)) = seq!(
        "[[",
        opt((take_until(0.., '|'), take(1usize))),
        (take_until(1.., "]]"), take(2usize))
    )
    .parse_next(input)?;

    let display_text = link_statement.1.map(|l| String::from(l.0));
    let link = link_statement.2.0.to_string();

    Ok(Inline::Link { display_text, link })
}

fn external_link(input: &mut &str) -> ModalResult<Inline> {
    let link_statement: (&str, Option<(&str, &str)>, (&str, &str)) = seq!(
        "[ext[",
        opt((take_until(0.., '|'), take(1usize))),
        (take_until(1.., "]]"), take(2usize))
    )
    .parse_next(input)?;

    let display_text = link_statement.1.map(|l| String::from(l.0));
    let link = link_statement.2.0.to_string();

    Ok(Inline::Link { display_text, link })
}

fn dimensions(input: &mut &str) -> ModalResult<Dimensions> {
    let fields: Vec<DimensionField> =
        separated(0.., alt((width, height)), space1).parse_next(input)?;

    let mut dims = Dimensions::default();

    for f in fields {
        match f {
            DimensionField::Height(h) => dims.height = Some(h),
            DimensionField::Width(w) => dims.width = Some(w),
        }
    }

    Ok(dims)
}

fn width(input: &mut &str) -> ModalResult<DimensionField> {
    ("width=", digit1)
        .map(|f: (&str, &str)| DimensionField::Width(f.1.to_string()))
        .parse_next(input)
}

fn height(input: &mut &str) -> ModalResult<DimensionField> {
    ("height=", digit1)
        .map(|f: (&str, &str)| DimensionField::Height(f.1.to_string()))
        .parse_next(input)
}

fn embedded_backticks() {
    todo!()
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

fn formatting(input: &mut &str) -> ModalResult<Inline> {
    alt((
        italics,
        bold,
        underlined,
        superscript,
        strikethrough,
        code,
        highlight,
    ))
    .parse_next(input)
}

fn inline(input: &mut &str) -> ModalResult<Inline> {
    alt((
        blockquote, codeblock, heading, link, image, formatting, plain_text,
    ))
    .parse_next(input)
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

    #[test]
    fn test_superscript() {
        let mut text = "This contains superscripted text as in y = x^^2^^.";
        let r = parse_wiki_text(&mut text);

        assert!(r.is_ok());

        let v = r.unwrap();
        assert_eq!(v.len(), 3);

        let expected: Vec<Inline> = vec![
            Inline::plaintext("This contains superscripted text as in y = x"),
            Inline::superscript("2"),
            Inline::plaintext("."),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_strikethrough() {
        let mut text = "This ~~is correct~~ is wrong.";
        let r = parse_wiki_text(&mut text);

        assert!(r.is_ok());

        let v = r.unwrap();
        assert_eq!(v.len(), 3);

        let expected: Vec<Inline> = vec![
            Inline::plaintext("This "),
            Inline::strikethrough("is correct"),
            Inline::plaintext(" is wrong."),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_code() {
        let mut text = "This `variable` is set to 0.";
        let r = parse_wiki_text(&mut text);

        assert!(r.is_ok());

        let v = r.unwrap();
        assert_eq!(v.len(), 3);

        let expected: Vec<Inline> = vec![
            Inline::plaintext("This "),
            Inline::code("variable"),
            Inline::plaintext(" is set to 0."),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_highlight() {
        let mut text = "@@Important point@@ should be noted.";
        let r = parse_wiki_text(&mut text);

        assert!(r.is_ok());

        let v = r.unwrap();
        assert_eq!(v.len(), 2);

        let expected: Vec<Inline> = vec![
            Inline::highlight("Important point"),
            Inline::plaintext(" should be noted."),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_block_quote_direct() {
        let mut text = "<<< 
        Knowing yourself is the beginning of all wisdom.
        <<< Aristotle
        ";
        let r = blockquote(&mut text);

        assert!(r.is_ok());

        let inline = r.unwrap();
        // assert_eq!(v.len(), 2);

        assert_eq!(
            inline,
            Inline::blockquote(
                "Knowing yourself is the beginning of all wisdom.",
                "Aristotle"
            )
        );
    }

    #[test]
    fn test_block_quote() {
        // let mut text = "A word of wisdom:
        // <<<
        // Knowing yourself is the beginning of all wisdom.
        // <<< Aristotle
        // ";

        let mut text = concat!(
            "A word of wisdom:\n",
            "<<<\n",
            "Knowing yourself is the beginning of all wisdom.\n",
            "<<< Aristotle"
        );

        let r = parse_wiki_text(&mut text);

        assert!(r.is_ok());

        let v = r.unwrap();
        // assert_eq!(v.len(), 2);

        let expected: Vec<Inline> = vec![
            Inline::plaintext("A word of wisdom:\n"),
            Inline::blockquote(
                "Knowing yourself is the beginning of all wisdom.",
                "Aristotle",
            ),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_block_quote_without_citation() {
        let mut text = concat!(
            "A word of wisdom:\n",
            "<<<\n",
            "Knowing yourself is the beginning of all wisdom.\n",
            "<<< "
        );

        let r = parse_wiki_text(&mut text);

        assert!(r.is_ok());

        let v = r.unwrap();
        // assert_eq!(v.len(), 2);

        let expected: Vec<Inline> = vec![
            Inline::plaintext("A word of wisdom:\n"),
            Inline::blockquote("Knowing yourself is the beginning of all wisdom.", ""),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_code_block_direct() {
        let expected_code = concat!("    let flag = x <= 5;\n", "    println!(\"{}\", flag);\n",);

        let mut text = String::from("```rust\n");
        text.push_str(expected_code);
        text.push_str("```");

        let r = codeblock(&mut text.as_str());

        assert!(r.is_ok());

        let inline = r.unwrap();

        assert_eq!(inline, Inline::codeblock(expected_code, "rust"));
    }

    #[test]
    fn test_code_block() {
        let expected_code = "let x = function();\n";

        let mut text = String::from("The code is:\n");
        text.push_str("```rust\n");
        text.push_str(expected_code);
        text.push_str("```\n");

        let r = parse_wiki_text(&mut text.as_str());

        assert!(r.is_ok());

        let v = r.unwrap();
        //assert_eq!(v.len(), 2);

        let expected: Vec<Inline> = vec![
            Inline::plaintext("The code is:\n"),
            Inline::codeblock(expected_code, "rust"),
            Inline::plaintext("\n"),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_heading_direct() {
        let mut text = "! Header Level 1";

        let r = heading(&mut text);

        assert!(r.is_ok());

        let inline = r.unwrap();

        assert_eq!(inline, Inline::heading("Header Level 1", 1));

        let mut text = "!!! Header Level 3";

        let r = heading(&mut text);

        assert!(r.is_ok());

        let inline = r.unwrap();

        assert_eq!(inline, Inline::heading("Header Level 3", 3));

        let mut text = "!!!!!! Header Level 6";

        let r = heading(&mut text);

        assert!(r.is_ok());

        let inline = r.unwrap();

        assert_eq!(inline, Inline::heading("Header Level 6", 6));
    }

    #[test]
    #[should_panic]
    fn test_heading_direct_boundary_conditions() {
        let mut text = " Some text!  Really";

        let inline = heading(&mut text).unwrap();
    }

    #[test]
    fn test_heading() {
        let mut text = "!! The End!\nBody text.";

        let v = parse_wiki_text(&mut text).unwrap();

        let expected: Vec<Inline> = vec![
            Inline::heading("The End!", 2),
            Inline::plaintext("\nBody text."),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_heading_with_leading_spaces() {
        let mut text = "   !! The End!\nBody text.";

        let v = parse_wiki_text(&mut text).unwrap();

        let expected: Vec<Inline> = vec![
            Inline::plaintext("   "),
            Inline::heading("The End!", 2),
            Inline::plaintext("\nBody text."),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_image_width() {
        let mut text = "width=32";
        let w = width(&mut text).unwrap();
        assert_eq!(w, DimensionField::Width(String::from("32")));
    }

    #[test]
    fn test_image_height() {
        let mut text = "height=20";
        let w = height(&mut text).unwrap();
        assert_eq!(w, DimensionField::Height(String::from("20")));
    }

    #[test]
    fn test_image_dimensions() {
        let mut text = "width=32 height=20";
        let d = dimensions(&mut text).unwrap();

        let expected = Dimensions {
            width: Some(String::from("32")),
            height: Some(String::from("20")),
        };

        assert_eq!(d, expected);

        let mut text = "height=20 width=32";
        let d = dimensions(&mut text).unwrap();
        assert_eq!(d, expected);

        let mut text = "height=20     width=32    ";
        let d = dimensions(&mut text).unwrap();
        assert_eq!(d, expected);
    }

    #[test]
    fn test_image_direct_simple() {
        let mut text = "[img[my picture.jpg]]";

        let inline = image(&mut text).unwrap();

        assert_eq!(inline, Inline::image("my picture.jpg", "", "", ""));
    }

    #[test]
    fn test_image_direct_with_width_dimension() {
        let mut text = "[img width=20 [my picture.jpg]]";

        let inline = image(&mut text).unwrap();

        assert_eq!(inline, Inline::image("my picture.jpg", "", "20", ""));
    }

    #[test]
    fn test_image_direct_with_height_dimension() {
        let mut text = "[img height=32   [my picture.jpg]]";

        let inline = image(&mut text).unwrap();

        assert_eq!(inline, Inline::image("my picture.jpg", "", "", "32"));
    }

    #[test]
    fn test_image_direct_with_caption() {
        let mut text = "[img[A Caption|my picture.jpg]]";

        let inline = image(&mut text).unwrap();

        assert_eq!(inline, Inline::image("my picture.jpg", "A Caption", "", ""));
    }

    #[test]
    fn test_image_direct_with_caption_and_dimensions() {
        let mut text = "[img width=32 height=20 [A Caption|my picture.jpg]]";

        let inline = image(&mut text).unwrap();

        assert_eq!(
            inline,
            Inline::image("my picture.jpg", "A Caption", "32", "20")
        );
    }

    #[test]
    fn test_image() {
        let mut text = "This is a picture [img width=32 height=20 [A Caption|my picture.jpg]] that is green coloured.";

        let v = parse_wiki_text(&mut text).unwrap();

        let expected = vec![
            Inline::plaintext("This is a picture "),
            Inline::image("my picture.jpg", "A Caption", "32", "20"),
            Inline::plaintext(" that is green coloured."),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_link_direct() {
        let mut text = "[[A Link]]";

        let v = link(&mut text).unwrap();

        assert_eq!(v, Inline::link("A Link", ""));
    }

    #[test]
    fn test_link_direct_with_caption() {
        let mut text = "[[This is the display text|A Link]]";

        let v = link(&mut text).unwrap();

        assert_eq!(v, Inline::link("A Link", "This is the display text"));
    }

    #[test]
    fn test_external_link_direct() {
        let mut text = "[ext[An external Link]]";

        let v = link(&mut text).unwrap();

        assert_eq!(v, Inline::link("An external Link", ""));
    }
}
