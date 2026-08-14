use std::fmt::Display;

/// Obsidian markdown generation
///
///
use crate::abstract_syntax::{CellAlignment, CellHorizontalAlignment, Inline, TableCell, TableRow};

pub struct Markdown(String);

impl Markdown {
    fn new() -> Self {
        Self(String::new())
    }

    pub fn from_inlines(inlines: &Vec<Inline>) -> Self {
        let mut markdown = Markdown::new();
        for inline in inlines {
            markdown.append(inline);
        }
        markdown
    }

    fn append(&mut self, inline: &Inline) {
        let s = self.convert_inline(inline);
        self.0.push_str(&s);
    }

    fn convert_inline(&mut self, inline: &Inline) -> String {
        match inline {
            Inline::PlainText { text } => text.to_string(),
            Inline::Italics { text } => Self::wrap("__", text),
            Inline::Bold { text } => Self::wrap("**", text),

            Inline::Underlined { text } => format!("<u>{text}</u>"),
            Inline::Superscript { text } => format!("<sup>{text}</sup>"),
            Inline::Subscript { text } => format!("<sub>{text}</sub>"),
            Inline::Strikethrough { text } => Self::wrap("~~", text),
            Inline::Code { text } => Self::wrap("```", text),
            Inline::Highlight { text } => Self::wrap("==", text),
            Inline::BlockQuote { text, citation } => Self::format_blockquote(text, citation),

            Inline::CodeBlock { text, language } => format!(
                "```{}\n{}\n```",
                language.clone().unwrap_or(String::new()),
                text
            ),
            Inline::Heading { text, level } => format!("{} {}", "#".repeat(*level), text),
            Inline::Image {
                width,
                height,
                caption,
                link,
            } => Self::format_image(link, caption, width, height),
            Inline::Link {
                display_text,
                link,
                external,
            } => Self::format_link(link, display_text, external),

            Inline::OrderedList { level, numbering } => {
                format!(
                    "{}{}. ",
                    "\t".repeat(*level - 1),
                    numbering[*level - 1].to_string()
                )
            }
            Inline::UnorderedList { level } => format!("{}- ", "\t".repeat(level - 1)),
            Inline::Transclusion { tiddler } => format!("![[{tiddler}]]"),
            Inline::Table { rows } => Self::format_table(rows),
            Inline::EndOfText => String::new(),
            _ => String::new(),
        }
    }

    fn wrap(marker: &str, text: &str) -> String {
        let mut s = String::new();
        s.push_str(marker);
        s.push_str(text);
        s.push_str(marker);
        s
    }

    fn format_link(link: &String, display_text: &Option<String>, external: &bool) -> String {
        let mut markdown = String::new();

        if !external {
            // Process internal links
            if let Some(display_text) = display_text {
                markdown.push_str(&format!("[[{}|{}]]", link, display_text));
            } else {
                markdown.push_str(&format!("[[{}]]", link));
            }
        } else {
            // Process external links
            if let Some(display_text) = display_text {
                markdown.push_str(&format!("[{display_text}]({link})"));
            } else {
                markdown.push_str(&format!("[{}]({})", link, link));
            }
        }

        markdown
    }

    fn format_blockquote(text: &str, citation: &Option<String>) -> String {
        let citation = citation.clone().unwrap_or(String::new());
        format!("> {}\n\n   __{}__", text, citation)
    }

    fn format_image(
        link: &str,
        caption: &Option<String>,
        width: &Option<String>,
        height: &Option<String>,
    ) -> String {
        let caption = (caption.clone()).unwrap_or(link.to_string());
        let dimensions = match (width, height) {
            (Some(width), Some(height)) => format!("|{width}x{height}"),
            (Some(width), None) => format!("|{width}"),
            _ => String::new(),
        };
        format!("![{caption}{dimensions}]({link})")
    }

    fn format_table(rows: &Vec<TableRow>) -> String {
        let mut markdown = String::new();

        let mut is_header = false;

        for (n, row) in rows.iter().enumerate() {
            let mut cell_widths = Vec::new();
            for cell in row.cells.iter() {
                let cell_contents = Self::from_inlines(&cell.contents).to_string();
                let cell_contents = Self::align_table_cell(cell.alignment.clone(), &cell_contents);
                is_header = n == 0 && cell.header;

                if is_header {
                    cell_widths.push(cell.contents.len());
                };
                markdown.push_str(&format!("|{cell_contents}"));
            }
            markdown.push_str("|\n");

            if is_header {
                // Add a seperate line with the underlining, e.g
                // | ------ | ------ |
                let header_line = cell_widths.iter().fold(String::new(), |s, &w| {
                    format!("| {} ", "-".repeat(w.max(2)))
                });
                markdown.push_str(&header_line);
                markdown.push_str("|\n");
                is_header = false;
            };
        }
        markdown
    }

    fn align_table_cell(alignment: CellAlignment, contents: &str) -> String {
        match alignment.horizontal {
            CellHorizontalAlignment::Left => format!("{contents} "),
            CellHorizontalAlignment::Right => format!(" {contents}"),
            CellHorizontalAlignment::Center => format!(" {contents} "),
        }
    }
}

impl Display for Markdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::markdown;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_from_formatting_inline() {
        let test_vector = vec![
            (Inline::plaintext("Test text"), "Test text"),
            (Inline::italics("Test text"), "__Test text__"),
            (Inline::bold("Test text"), "**Test text**"),
            (Inline::underlined("Test text"), "<u>Test text</u>"),
            (Inline::superscript("Test text"), "<sup>Test text</sup>"),
            (Inline::subscript("Test text"), "<sub>Test text</sub>"),
            (Inline::strikethrough("Test text"), "~~Test text~~"),
            (Inline::code("Test text"), "```Test text```"),
            (Inline::highlight("Test text"), "==Test text=="),
        ];

        for v in test_vector {
            let mut markdown = Markdown::new();

            markdown.append(&v.0);
            assert_eq!(markdown.to_string(), v.1);
        }
    }

    #[test]
    fn test_blockquote() {
        let mut markdown = Markdown::new();
        markdown.append(&Inline::blockquote("This is a quote", "Citation"));

        assert_eq!(markdown.to_string(), "> This is a quote\n\n   __Citation__");
    }

    #[test]
    fn test_codeblock() {
        let mut markdown = Markdown::new();
        markdown.append(&Inline::codeblock("let x = 0;", "rust"));

        assert_eq!(markdown.to_string(), "```rust\nlet x = 0;\n```");
    }

    #[test]
    fn test_heading() {
        let mut markdown = Markdown::new();
        markdown.append(&Inline::heading("Title", 1));
        assert_eq!(markdown.to_string(), "# Title");

        let mut markdown = Markdown::new();
        markdown.append(&Inline::heading("Subheading", 3));
        assert_eq!(markdown.to_string(), "### Subheading");
    }

    #[test]
    fn test_image() {
        let mut markdown = Markdown::new();
        markdown.append(&Inline::image("image", "Title", "200", "100"));
        assert_eq!(markdown.to_string(), "![Title|200x100](image)");

        let mut markdown = Markdown::new();
        markdown.append(&Inline::image("image", "Title", "200", ""));
        assert_eq!(markdown.to_string(), "![Title|200](image)");

        let mut markdown = Markdown::new();
        markdown.append(&Inline::image("image", "Title", "", ""));
        assert_eq!(markdown.to_string(), "![Title](image)");
    }

    #[test]
    fn test_link() {
        let mut markdown = Markdown::new();
        markdown.append(&Inline::link("link", "Title"));
        assert_eq!(markdown.to_string(), "[[link|Title]]");

        let mut markdown = Markdown::new();
        markdown.append(&Inline::link("link", ""));
        assert_eq!(markdown.to_string(), "[[link]]");
    }

    #[test]
    fn test_ext_link() {
        let mut markdown = Markdown::new();
        markdown.append(&Inline::ext_link("link", "Title"));
        assert_eq!(markdown.to_string(), "[Title](link)");

        let mut markdown = Markdown::new();
        markdown.append(&Inline::ext_link("link", ""));
        assert_eq!(markdown.to_string(), "[link](link)");
    }

    #[test]
    fn test_ordered_list() {
        let inlines = vec![
            Inline::plaintext("The following points:\n"),
            Inline::ordered_list(1, "1"),
            Inline::plaintext("The most important\n"),
            Inline::ordered_list(1, "2"),
            Inline::plaintext("Not so important\n"),
            Inline::ordered_list(2, "2.1"),
            Inline::plaintext("Why this is not important"),
        ];

        let expected = concat!(
            "The following points:\n",
            "1. The most important\n",
            "2. Not so important\n",
            "\t1. Why this is not important"
        );

        let markdown = Markdown::from_inlines(&inlines);

        let contents = markdown.to_string();
        assert_eq!(contents, expected);
    }

    #[test]
    fn test_unordered_list() {
        let inlines = vec![
            Inline::plaintext("The following points:\n"),
            Inline::unordered_list(1),
            Inline::plaintext("The most important\n"),
            Inline::unordered_list(1),
            Inline::plaintext("Not so important\n"),
            Inline::unordered_list(2),
            Inline::plaintext("Why this is not important"),
        ];

        let expected = concat!(
            "The following points:\n",
            "- The most important\n",
            "- Not so important\n",
            "\t- Why this is not important"
        );

        let markdown = Markdown::from_inlines(&inlines);

        let contents = markdown.to_string();
        assert_eq!(contents, expected);
    }

    #[test]
    fn test_transclusion() {
        let inlines = vec![Inline::transclusion("link")];

        let expected = "![[link]]";

        let markdown = Markdown::from_inlines(&inlines);

        let contents = markdown.to_string();
        assert_eq!(contents, expected);
    }

    #[test]
    fn test_table() {
        //     Table {
        //     rows: Vec<TableRow>,
        // }
        let row1: Vec<TableCell> = vec![
            TableCell::new("Column 1").header().center().build(),
            TableCell::new("Column 2").header().center().build(),
            TableCell::new("Column 3").header().center().build(),
        ];

        let row2: Vec<TableCell> = vec![
            TableCell::new("left").left().build(),
            TableCell::new("center").center().build(),
            TableCell::new("right").right().build(),
        ];

        let row3: Vec<TableCell> = vec![
            TableCell::new_with_inlines(vec![Inline::bold("bold")])
                .center()
                .build(),
            TableCell::new_with_inlines(vec![Inline::italics("italic")])
                .center()
                .build(),
            TableCell::new_with_inlines(vec![Inline::link("link", "")])
                .center()
                .build(),
        ];

        let rows: Vec<TableRow> = vec![
            TableRow::new(row1),
            TableRow::new(row2),
            TableRow::new(row3),
        ];
        let inlines = vec![Inline::Table { rows }];

        let expected = concat!(
            "| Column 1 | Column 2 | Column 3 |\n",
            "| -- |\n",
            "|left | center | right|\n",
            "| **bold** | __italic__ | [[link]] |\n",
        );

        let markdown = Markdown::from_inlines(&inlines);

        let contents = markdown.to_string();
        assert_eq!(contents, expected);
    }
}
