/// Parses the WikiText as defined [here](https://tiddlywiki.com/static/WikiText.html)
use log::{Level, debug, error, info, log_enabled};
use std::assert_matches;

use winnow::{
    ModalResult, Parser,
    ascii::{
        alphanumeric0, alphanumeric1, digit1, line_ending, multispace0, multispace1, space0,
        space1, tab,
    },
    combinator::{
        alt, delimited, dispatch, eof, fail, not, opt, preceded, repeat, repeat_till, separated,
        seq, terminated,
    },
    error::StrContext,
    stream::AsChar,
    token::{any, literal, one_of, take, take_till, take_until, take_while},
};

use crate::abstract_syntax::{
    CellAlignment, CellHorizontalAlignment, CellVerticalAlignment, DimensionField, Dimensions,
    Inline::{self, PlainText},
    TableCell, TableRow,
};

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
        take_until(1.., marker),
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

fn unordered_list(input: &mut &str) -> ModalResult<Inline> {
    // 1. Count the number of asterixs (1 to 6) to determine the list level
    let hashes: Vec<char> = repeat(1..=6, one_of('*')).parse_next(input)?;
    let level = hashes.len();

    // 2. Consume the required trailing whitespace separating the * and the text
    let _space = space1.parse_next(input)?;

    // 3. Consume everything else on the line as the list item text
    let text = take_till(0.., |c| c == '\n' || c == '\r').parse_next(input)?;

    Ok(Inline::UnorderedList {
        level,
        text: text.to_string(),
    })
}

fn ordered_list(input: &mut &str) -> ModalResult<Inline> {
    // 1. Count the number of asterixs (1 to 6) to determine the list level
    let hashes: Vec<char> = repeat(1..=6, one_of('#')).parse_next(input)?;
    let level = hashes.len();

    // 2. Consume the required trailing whitespace separating the # and the text
    let _space = space1.parse_next(input)?;

    // 3. Consume everything else on the line as the list item text
    let text = take_till(0.., |c| c == '\n' || c == '\r').parse_next(input)?;

    Ok(Inline::OrderedList {
        level,
        text: text.to_string(),
    })
}

// table = {table_row}*;
// table_row = {white_space}, "|",  {table_cell} , {white_space} (end_of_line | eof);
// table_cell = [[pre-alignment] , [white_space] , [header] , [table_cell_contents], [post-alignment] '|';
// table_cell_contents = {alphanumeric | white_space | formatting | link};
// header = '!';
// pre-alignment = ' ' | '^' | ',' , {whitespace};
// post-alignment = {whitespace}*
//
// * = 1 or more
//

fn vertical_alignment(input: &mut &str) -> ModalResult<CellAlignment> {
    let c = take(1usize).parse_next(input)?;

    let mut alignment = CellAlignment {
        ..Default::default()
    };

    match c {
        "^" => alignment.vertical = CellVerticalAlignment::Top,
        "," => alignment.vertical = CellVerticalAlignment::Bottom,
        _ => fail.parse_next(input)?,
    }

    Ok(alignment)
}

fn plaintext(input: &mut &str) -> ModalResult<Inline> {
    // let s = alt((alphanumeric1, multispace1)).parse_next(input)?;
    let german_letters = ['ß', 'ä', 'ö', 'ü'];
    //let s = take_while(0.., |c| {
    let s = take_while(1.., |c| {
        AsChar::is_alphanum(c) || AsChar::is_space(c) || german_letters.contains(&c)
    })
    .parse_next(input)?;

    Ok(Inline::PlainText {
        text: s.to_string(),
    })
}

// #[deprecated]
// fn table_cell_contents(input: &mut &str) -> ModalResult<Vec<Inline>> {
//     let v = repeat(0.., alt((formatting, link, image, plaintext)))
//         .fold(Vec::new, |mut acc: Vec<_>, item| {
//             acc.push(item);
//             acc
//         })
//         .parse_next(input)?;

//     Ok(v)
// }

fn table_cell(input: &mut &str) -> ModalResult<TableCell> {
    //debug!("Entering table_cell parser with: {}", input);

    let (alignment, leading_spaces, header, mut contents, _) = seq!(
        opt(vertical_alignment),
        space0,
        opt("!"),
        // table_cell_contents,
        take_until(1.., '|'),
        take(1usize) // remove the trailng '|'
    )
    .parse_next(input)?;

    // debug!(
    //     "alignment = {:?}, leading_spaces = {:?}, header = {:?}, contents = {:?}",
    //     alignment.clone(),
    //     leading_spaces,
    //     header,
    //     contents
    // );

    // Parse the contents and convert into Inline(s) by recursively calling this.
    let s = contents.to_owned();
    let mut contents = parse_wiki_text(&mut s.as_str())?;

    let mut alignment = alignment.map_or(CellAlignment::default(), |a| a);

    // Handle the horizonal alignment seperately by looking for spaces at
    // the start and end of the contents. Two complications:
    // 1)  the leading spaces are consumed by the parser, so use the value returned by it and check the last character
    // 2) The contents are really inlines, so need to find the last one.
    let start_space = !leading_spaces.is_empty();
    let end_space = if let Some(Inline::PlainText { text }) = contents.last() {
        text.chars().next_back().map_or(false, |c| c == ' ')
    } else {
        false
    };

    // let end_space = contents.chars().next_back().map_or(false, |c| c == ' ');

    let horizontal_alignment = match (start_space, end_space) {
        (true, true) => CellHorizontalAlignment::Center,
        (true, false) => CellHorizontalAlignment::Right,
        (false, true) => CellHorizontalAlignment::Left,
        _ => CellHorizontalAlignment::default(),
    };
    alignment.horizontal = horizontal_alignment;

    let header = header.map_or(false, |s| s == "!");

    // Trim single spaces at the end of the content as these only refer to the alignment.
    // Any aligment space at the start has been consumed by the parser

    if let Some(Inline::PlainText { text }) = contents.last() {
        let trimmed_text = trim_single_trailing_whitespace(text.clone());
        // Remove the last plain text element of the inlines and replace it
        // with the trimmed version
        contents.pop();
        contents.push(PlainText { text: trimmed_text });
    }

    // TODO not doing merges at the moment

    let table_cell = TableCell {
        contents,
        alignment,
        header,
        ..Default::default()
    };

    Ok(table_cell)
}

fn table_row(input: &mut &str) -> ModalResult<TableRow> {
    // debug!("Entering table_row parser with :  {}", input);

    "|".parse_next(input)?;
    let cells = repeat_till(1.., table_cell, alt((line_ending, eof)))
        .map(|v: (Vec<TableCell>, &str)| v.0)
        .parse_next(input)?;

    // debug!("Table row cells:  {:?}", cells);

    Ok(TableRow { cells })
}

fn table(input: &mut &str) -> ModalResult<Inline> {
    //let v = repeat_till(1.., table_row, not(table_row))
    let v = repeat(1.., table_row)
        .fold(Vec::new, |mut acc: Vec<_>, item| {
            acc.push(item);
            acc
        })
        .parse_next(input)?;

    Ok(Inline::Table { rows: v })
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
    let link = alt((camel_case_link, simple_link, external_link)).parse_next(input)?;

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

fn camel_case_link(input: &mut &str) -> ModalResult<Inline> {
    // A camel case link is at least two humps back-to-back
    let l = (hump, repeat(1.., hump).map(|_: Vec<_>| ()))
        .take()
        .parse_next(input)?;

    Ok(Inline::Link {
        display_text: None,
        link: l.to_string(),
    })
}

// one "hump": an uppercase letter followed by 1+ lowercase/digit chars
fn hump<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    (
        take_while(1, |c: char| c.is_ascii_uppercase()),
        take_while(1.., |c: char| c.is_ascii_lowercase() || c.is_ascii_digit()),
    )
        .take()
        .parse_next(input)
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
// Only handling simple transclusion
// TODO
fn transclusion(input: &mut &str) -> ModalResult<Inline> {
    let transclusion_statement =
        seq!("{{", take_until(1.., "}}"), take(2usize)).parse_next(input)?;

    let transclusion = transclusion_statement.1.to_string();

    Ok(Inline::Transclusion {
        tiddler: transclusion,
    })
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

fn list(input: &mut &str) -> ModalResult<Inline> {
    alt((unordered_list, ordered_list)).parse_next(input)
}

// Only adding the intermediate parser so that the alt in the inline
// parser does not become too long
fn blocks(input: &mut &str) -> ModalResult<Inline> {
    alt((blockquote, codeblock)).parse_next(input)
}

fn inline(input: &mut &str) -> ModalResult<Inline> {
    alt((
        blocks,
        heading,
        list,
        link,
        transclusion,
        image,
        formatting,
        table,
        end_of_text, //plain_text,
    ))
    .parse_next(input)
}

pub fn end_of_text(input: &mut &str) -> ModalResult<Inline> {
    eof.parse_next(input)?;

    Ok(Inline::EndOfText)
}

pub fn parse_wiki_text(input: &mut &str) -> ModalResult<Vec<Inline>> {
    let mut inlines = Vec::<Inline>::new();

    debug!("Entering parse_wiki_text with: {}", input);

    while !input.is_empty() {
        // let v = repeat_till(0.., any, inline)
        //     .map(|v: (Vec<char>, Inline)| v)
        //     .with_taken()
        //     .parse_next(input)?
        //     .0;

        // let ((characters, inline), _) = repeat_till(0.., any, inline)
        //     .map(|v: (Vec<char>, Inline)| v)
        //     .with_taken()
        //     .parse_next(input)?;

        // let s: String = characters.into_iter().collect();

        let (characters, inline) = repeat_till(0.., any, inline)
            .map(|v: (Vec<char>, Inline)| v)
            .parse_next(input)?;

        let s: String = characters.into_iter().collect();

        debug!("parse_wiki_text s={}", s);

        // Sometimes a parser produces an empty plain text entry.
        // Filter these out
        if !s.is_empty() {
            inlines.push(Inline::PlainText { text: s });
        };
        debug!("parse_wiki_text inlines: {:?}", inlines);

        // Filter out end of text as not needed
        if inline != Inline::EndOfText {
            inlines.push(inline)
        };
    }

    Ok(inlines)
}

// Helper function
fn trim_single_trailing_whitespace(mut s: String) -> String {
    if matches!(s.chars().last(), Some(' ') | Some('\t')) {
        s.pop();
    }
    s
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};
    use serde_json::ser::CharEscape::Tab;

    use super::*;

    fn log_this() {
        simple_logger::init_with_level(log::Level::Debug).unwrap();
    }

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
            Inline::plaintext("A word of wisdom:"),
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
            Inline::plaintext("A word of wisdom:"),
            Inline::blockquote("Knowing yourself is the beginning of all wisdom.", ""),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_block_quote_multiline() {
        // let mut text = "A word of wisdom:
        // <<<
        // Knowing yourself is the beginning of all wisdom.
        // <<< Aristotle
        // ";

        let mut text = concat!(
            "A word of wisdom:\n",
            "<<<\n",
            "The saddest aspect of life right now is that science\n",
            "gathers knowledge faster than society gathers wisdom.\n",
            "<<< Isaac Asimov"
        );

        let r = parse_wiki_text(&mut text);

        assert!(r.is_ok());

        let v = r.unwrap();
        // assert_eq!(v.len(), 2);

        let expected: Vec<Inline> = vec![
            Inline::plaintext("A word of wisdom:"),
            Inline::blockquote(
                "The saddest aspect of life right now is that science\ngathers knowledge faster than society gathers wisdom.",
                "Isaac Asimov",
            ),
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
    fn test_mixed_links() {
        let mut input =
            "This is an inline [[link]] with some more text and a [[another link|new link]]";
        let v = parse_wiki_text(&mut input).unwrap();

        let expect = vec![
            Inline::plaintext("This is an inline "),
            Inline::link("link", ""),
            Inline::plaintext(" with some more text and a "),
            Inline::link("new link", "another link"),
        ];

        assert_eq!(v, expect);
    }

    #[test]
    fn test_external_link_direct() {
        let mut text = "[ext[An external Link]]";

        let v = link(&mut text).unwrap();

        assert_eq!(v, Inline::link("An external Link", ""));
    }

    #[test]
    fn test_hump_direct() {
        let mut text = "CamelCase";
        let s = hump(&mut text).unwrap();
        assert_eq!(s, "Camel");

        let mut text = "CamelCaseDouble";
        let s = hump(&mut text).unwrap();
        assert_eq!(s, "Camel");

        let mut text = "plaintext";
        assert!(hump(&mut text).is_err());
    }

    #[test]
    fn test_camel_case_link_direct() {
        let mut text = "CamelCase";

        let inline = camel_case_link(&mut text).unwrap();

        assert_eq!(
            inline,
            Inline::Link {
                display_text: None,
                link: "CamelCase".to_string()
            }
        );

        let mut text = "CamelCaseAgain";

        let inline = camel_case_link(&mut text).unwrap();

        assert_eq!(
            inline,
            Inline::Link {
                display_text: None,
                link: "CamelCaseAgain".to_string()
            }
        );
    }

    #[test]
    // #[ignore = "Currently does not work."]
    fn test_camel_case_link() {
        let mut text =
            "A key capability of WikiText is the ability to make links, even CamelCaseLinks.";

        let v = parse_wiki_text(&mut text).unwrap();

        let expected = vec![
            Inline::plaintext("A key capability of "),
            Inline::link("WikiText", ""),
            Inline::plaintext(" is the ability to make links, even "),
            Inline::link("CamelCaseLinks", ""),
            Inline::plaintext("."),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_links_embedded() {
        log_this();
        let mut input = "This is some [[link]] in the middle of text followed by [[another link]].\n//italics//";

        let l = parse_wiki_text(&mut input).unwrap();

        let expectations = vec![
            Inline::plaintext("This is some "),
            Inline::link("link", ""),
            Inline::plaintext(" in the middle of text followed by "),
            Inline::link("another link", ""),
            Inline::plaintext(".\n"),
            Inline::italics("italics"),
        ];

        assert_eq!(l, expectations);
    }

    #[test]
    fn test_end_of_text() {
        let mut text = "";

        let inline = end_of_text(&mut text).unwrap();

        assert_eq!(inline, Inline::EndOfText);
    }

    #[test]
    fn test_parse_wiki_text() {
        let mut text = "Hello world\n ''bold''";

        let inlines = parse_wiki_text(&mut text).unwrap();

        let expected = vec![
            Inline::PlainText {
                text: "Hello world\n ".to_string(),
            },
            Inline::Bold {
                text: "bold".to_string(),
            },
        ];
        assert_eq!(inlines, expected);
    }

    #[test]
    fn test_unordered_list_direct() {
        let mut text = "* List Level 1";

        let r = unordered_list(&mut text);

        assert!(r.is_ok());

        let inline = r.unwrap();

        assert_eq!(inline, Inline::unordered_list("List Level 1", 1));

        let mut text = "*** List Level 3";

        let r = unordered_list(&mut text);

        assert!(r.is_ok());

        let inline = r.unwrap();

        assert_eq!(inline, Inline::unordered_list("List Level 3", 3));

        let mut text = "****** List Level 6";

        let r = unordered_list(&mut text);

        assert!(r.is_ok());

        let inline = r.unwrap();

        assert_eq!(inline, Inline::unordered_list("List Level 6", 6));
    }

    #[test]
    fn test_unordered_list() {
        let mut text = concat!(
            "The following points:\n",
            "* The most important\n",
            "* Not so important\n",
            "** Why this is not important"
        );

        let v = parse_wiki_text(&mut text).unwrap();

        let expected = vec![
            Inline::plaintext("The following points:\n"),
            Inline::unordered_list("The most important", 1),
            Inline::plaintext("\n"),
            Inline::unordered_list("Not so important", 1),
            Inline::plaintext("\n"),
            Inline::unordered_list("Why this is not important", 2),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_ordered_list() {
        let mut text = concat!(
            "The following points:\n",
            "# The most important\n",
            "# Not so important\n",
            "## Why this is not important"
        );

        let v = parse_wiki_text(&mut text).unwrap();

        let expected = vec![
            Inline::plaintext("The following points:\n"),
            Inline::ordered_list("The most important", 1),
            Inline::plaintext("\n"),
            Inline::ordered_list("Not so important", 1),
            Inline::plaintext("\n"),
            Inline::ordered_list("Why this is not important", 2),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_transclusion() {
        let mut text = concat!(
            "! How to do it\n",
            "{{Instructions}}",
            "!! What to avoid\n",
            "{{Avoid}}"
        );

        let v = parse_wiki_text(&mut text).unwrap();

        let expected = vec![
            Inline::heading("How to do it", 1),
            Inline::plaintext("\n"),
            Inline::transclusion("Instructions"),
            Inline::heading("What to avoid", 2),
            Inline::plaintext("\n"),
            Inline::transclusion("Avoid"),
        ];

        assert_eq!(v, expected);
    }

    #[test]
    fn test_vertical_alignment() {
        let mut text = r#"^Contents"#;
        let v = vertical_alignment(&mut text).unwrap();
        assert_eq!(v.vertical, CellVerticalAlignment::Top);

        text = ",Contents";
        let v = vertical_alignment(&mut text).unwrap();
        assert_eq!(v.vertical, CellVerticalAlignment::Bottom);
    }

    #[test]
    fn test_plain_text() {
        let mut text = " Contents42 ";
        let v = plaintext(&mut text).unwrap();
        assert_eq!(
            v,
            Inline::PlainText {
                text: " Contents42 ".to_string()
            }
        );

        let mut text = " Contents 42 |";
        let v = plaintext(&mut text).unwrap();
        assert_eq!(
            v,
            Inline::PlainText {
                text: " Contents 42 ".to_string()
            }
        );
    }

    #[test]
    fn test_table_cell() {
        let mut text = "aaa|bbb|ccc|";

        let v = table_cell(&mut text).unwrap();

        // assert_eq!(
        //     v,
        //     TableCell {
        //         text: "aaa".to_string(),
        //         ..Default::default()
        //     }
        // );

        assert_eq!(v, TableCell::new("aaa"));
    }

    #[test]
    fn test_table_cell_with_inlines() {
        let mut input = "aaa [[link]] //italics//|bbb|''bold''|";

        let v = table_cell(&mut input).unwrap();

        assert_eq!(
            v,
            TableCell::new_with_inlines(vec![
                Inline::plaintext("aaa "),
                Inline::link("link", ""),
                Inline::plaintext(" "),
                Inline::italics("italics"),
            ])
        )
    }

    #[test]
    fn test_table_row() {
        let mut text = "|aaa|bbb|ccc|\n";

        let v = table_row(&mut text).unwrap();

        assert_eq!(
            v,
            TableRow {
                cells: vec![
                    TableCell::new("aaa"),
                    TableCell::new("bbb"),
                    TableCell::new("ccc")
                ]
            }
        )
    }

    #[test]
    fn test_table_row_with_headers() {
        let mut text = "|!aaa|!bbb|!ccc|\n";

        let v = table_row(&mut text).unwrap();

        assert_eq!(
            v,
            TableRow {
                cells: vec![
                    TableCell::new("aaa").header().build(),
                    TableCell::new("bbb").header().build(),
                    TableCell::new("ccc").header().build(),
                ]
            }
        )
    }

    #[test]
    fn test_table_row_with_horizontal_alignment() {
        let mut text = "| aaa | bbb|ccc |\n";

        let v = table_row(&mut text).unwrap();

        assert_eq!(
            v.cells[0].alignment.horizontal,
            CellHorizontalAlignment::Center,
            " TableCell: {:?}",
            v.cells[0],
        );

        assert_eq!(
            v.cells[1].alignment.horizontal,
            CellHorizontalAlignment::Right,
            " TableCell:{:?}",
            v.cells[1]
        );
        assert_eq!(
            v.cells[2].alignment.horizontal,
            CellHorizontalAlignment::Left,
            " TableCell: {:?}",
            v.cells[2]
        );

        assert_eq!(
            v,
            TableRow {
                cells: vec![
                    TableCell::new("aaa").center().build(),
                    TableCell::new("bbb").right().build(),
                    TableCell::new("ccc").left().build(),
                ]
            }
        )
    }

    #[test]
    fn test_table_row_with_spaced_content() {
        let mut text = "| Centered words | Right aligned words|Left aligned words |\n";

        let v = table_row(&mut text).unwrap();

        assert_eq!(
            v.cells[0],
            TableCell::new("Centered words").center().build()
        );

        assert_eq!(
            v.cells[1],
            TableCell::new("Right aligned words").right().build()
        );
        assert_eq!(
            v.cells[2],
            TableCell::new("Left aligned words").left().build()
        );
    }

    #[test]
    fn test_table_row_with_vertical_alignment() {
        let mut text =
            "| Middle words |^ Top and right aligned words|,Bottom and left aligned words |\n";

        let v = table_row(&mut text).unwrap();

        assert_eq!(
            v.cells[0],
            TableCell::new("Middle words").middle().center().build()
        );
        assert_eq!(
            v.cells[1],
            TableCell::new("Top and right aligned words")
                .top()
                .right()
                .build()
        );

        assert_eq!(
            v.cells[2],
            TableCell::new("Bottom and left aligned words")
                .bottom()
                .left()
                .build()
        );
    }

    #[test]
    fn test_another_row() {
        let mut text = "|^top left |^ top center |^ top right|\n";

        let table = table_row(&mut text).unwrap();

        assert!(true);
    }

    #[test]
    fn test_table_direct() {
        let mut text = "|!Left | !Middle | !Right|\n|^top left |^ top center |^ top right|\n|middle left | middle center | middle right|\n|,bottom left |, bottom center |, bottom right|";

        let table = table(&mut text).unwrap();

        if let Inline::Table { rows } = table {
            assert_eq!(rows.len(), 4);

            let row = &rows[0];
            let cell = &row.cells[1];
            assert_eq!(*cell, TableCell::new("Middle").header().center().build());

            let row = &rows[1];
            let cell = &row.cells[2];
            assert_eq!(*cell, TableCell::new("top right").right().top().build());

            let row = &rows[3];
            let cell = &row.cells[0];
            assert_eq!(*cell, TableCell::new("bottom left").left().bottom().build());
        } else {
            assert!(false, "Result is not Inline:.Table")
        }
    }

    #[test]
    //#[ignore = "Not working yet - TODO Change parse_wiki_text to use plaintext"]
    fn test_table_in_situ() {
        log_this();

        let mut text = "This is some text with a [[link]] followed by a table:\n|!Left | !Middle | !Right|\n|^top left |^ top center |^ top right|\n|middle left | middle center | middle right|\n|,bottom left |, bottom center |, bottom right|";

        let inlines = parse_wiki_text(&mut text).unwrap();

        //assert_eq!(inlines.len(), 4);
        assert_eq!(inlines[0], Inline::plaintext("This is some text with a "));
        assert_eq!(inlines[1], Inline::link("link", ""));
        assert_eq!(inlines[2], Inline::plaintext(" followed by a table:\n"));

        assert_matches!(&inlines[4], Inline::Table {rows} if rows.len() == 4);
        assert_matches!(&inlines[4], Inline::Table {rows} if rows[0].cells[1] == TableCell::new("Middle").header().center().build());
        assert_matches!(&inlines[4], Inline::Table {rows} if rows[1].cells[2] == TableCell::new("top right").right().top().build());
        assert_matches!(&inlines[4], Inline::Table {rows} if rows[3].cells[0] == TableCell::new("bottom left").left().bottom().build());

        // let row = &rows[0];
        // let cell = &row.cells[1];
        // assert_eq!(*cell, TableCell::new("Middle").header().center().build());

        // let row = &rows[1];
        // let cell = &row.cells[2];
        // assert_eq!(*cell, TableCell::new("top right").right().top().build());

        // let row = &rows[3];
        // let cell = &row.cells[0];
        // assert_eq!(*cell, TableCell::new("bottom left").left().bottom().build());
    }

    #[test]
    fn try_parsing() {
        log_this();

        let mut input = "This is an inline [[link]] with some more text and a [[new link]]";
        // while !input.is_empty() {
        //     let x = repeat_till(0.., inline, eof)
        //         .map(|v: (Vec<Inline>, &str)| v)
        //         .parse_next(&mut input)
        //         .unwrap();
        //     debug!("Inlines:\n{:?}\n", x.0);
        //     debug!("str:{:?}\n", x.1);
        // }

        while !input.is_empty() {
            let (characters, inline) = repeat_till(0.., any, inline)
                .map(|v: (Vec<char>, Inline)| v)
                .parse_next(&mut input)
                .unwrap();

            let s: String = characters.iter().collect();
            debug!("s:{}", s);
            debug!("Inline:{:?}", inline);
        }

        assert!(true);
    }
}
