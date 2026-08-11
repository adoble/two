use std::fs::File;
use std::io::prelude::*;

use anyhow::Result;

mod tiddler;
use tiddler::Tiddler;

mod markdown;
use markdown::Markdown;

#[allow(unused_imports)]
use log::{debug, error, info, log_enabled};

use crate::abstract_syntax::{CellAlignment, CellHorizontalAlignment, Inline, TableRow};
use crate::parser::parse_wiki_text;

mod parser;

mod abstract_syntax;

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    info!("TiddlyWiki to Obsidian");

    let mut tw_file = File::open("./tiddlywiki/empty.html").unwrap();
    let mut tw_string = String::new();
    tw_file.read_to_string(&mut tw_string).unwrap();

    let json = extract_tiddler_json(&tw_string);

    //println!("{}", json.unwrap());

    let tiddlers: Vec<Tiddler> = serde_json::from_str(json.unwrap()).unwrap();

    let filtered_tiddlers: Vec<&Tiddler> = tiddlers
        .iter()
        .filter(|t| !t.title.starts_with("$:/"))
        .collect();

    println!("len: {}", filtered_tiddlers.len());

    for tiddler in filtered_tiddlers {
        create_tiddler_file(tiddler).unwrap();
    }
}

fn extract_tiddler_json(html: &str) -> Option<&str> {
    let marker = r#"class="tiddlywiki-tiddler-store""#;
    let start_idx = html.find(marker)?;
    let tag_close = html[start_idx..].find('>')? + start_idx + 1;
    let end_idx = html[tag_close..].find("</script>")? + tag_close;
    Some(html[tag_close..end_idx].trim())
}

fn create_tiddler_file(tiddler: &Tiddler) -> Result<()> {
    // TODO file name from cli
    let file_name = format!("./output/{}.md", tiddler.title);

    // Parse the tiddler text
    let mut input = tiddler.text.as_str();
    let ast = parse_wiki_text(&mut input).unwrap();
    let output_text = convert_to_markdown(&ast);
    todo!("TODO write out the markdown to the files");

    // let mut file = OpenOptions::new()
    //     .append(true)
    //     .open(file_name)
    //     .expect("cannot open file");
    let mut file = File::create(file_name).expect("Could not crate file");

    file.write_all(tiddler.text.as_bytes())
        .expect("write failed");

    Ok(())
}

fn convert_to_markdown(ast: &Vec<Inline>) -> String {
    let mut markdown = String::new();
    for inline in ast {
        match inline {
            Inline::PlainText { text } => markdown.push_str(text),
            Inline::Italics { text } => markdown.push_str(&wrap("_", text)),
            Inline::Bold { text } => markdown.push_str(&wrap("**", text)),

            Inline::Underlined { text } => markdown.push_str(&format!("<u>{text}</u>")),
            Inline::Superscript { text } => markdown.push_str(&format!("<sup>{text}</sup>")),
            Inline::Subscript { text } => markdown.push_str(&format!("<sub>{text}</sub>")),
            Inline::Strikethrough { text } => markdown.push_str(&wrap("~~", text)),
            Inline::Code { text } => markdown.push_str(&wrap("`", text)),
            Inline::Highlight { text } => markdown.push_str(&wrap("==", text)),
            Inline::BlockQuote { text, citation } => {
                markdown.push_str(&format!("> {text}"));
                if let Some(citation) = citation {
                    markdown.push_str(&format!("\n\n   __{citation}__"));
                }
            }
            Inline::CodeBlock { text, language } => markdown.push_str(&format!(
                "```{}\n{}\n```",
                language.clone().unwrap_or(String::new()),
                text
            )),
            Inline::Heading { text, level } => {
                for _ in 0..*level {
                    markdown.push('#')
                }
                markdown.push(' ');
                markdown.push_str(text);
            }
            Inline::Image {
                width,
                height,
                caption,
                link,
            } => {
                let caption = (caption.clone()).unwrap_or(link.clone());
                let dimensions = match (width, height) {
                    (Some(width), Some(height)) => format!("|{width}x{height}"),
                    (Some(width), None) => format!("|{width}"),
                    _ => String::new(),
                };
                markdown.push_str(&format!("![{caption}{dimensions}]({link})"));
            }
            Inline::Link {
                display_text,
                link,
                external,
            } => {
                markdown.push_str(&link_markdown(link, display_text, external));
            }
            Inline::OrderedList { text, level } => todo!(),
            Inline::UnorderedList { text, level } => {
                markdown.push_str(&format!("- {}{}", " ".repeat(level * 4), text))
            }
            Inline::Transclusion { tiddler } => markdown.push_str(&format!("![[{tiddler}]]")),
            Inline::Table { rows } => markdown.push_str(&table_markdown(rows)),
            Inline::EndOfText => (),
        }
    }

    markdown
}

fn wrap(marker: &str, text: &str) -> String {
    let mut s = String::new();
    s.push_str(marker);
    s.push_str(text);
    s.push_str(marker);
    s
}

fn link_markdown(link: &String, display_text: &Option<String>, external: &bool) -> String {
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

fn table_markdown(rows: &Vec<TableRow>) -> String {
    let mut markdown = String::new();

    let mut header = String::new();
    let mut is_header = false;

    for (n, row) in rows.iter().enumerate() {
        let mut cell_widths = Vec::new();
        for cell in row.cells.iter() {
            let cell_contents = convert_to_markdown(&cell.contents);
            let cell_contents = align_table_cell(cell.alignment.clone(), &cell_contents);
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
        CellHorizontalAlignment::Left => format!("{contents}  "),
        CellHorizontalAlignment::Right => format!("  {contents}"),
        CellHorizontalAlignment::Center => format!("  {contents}  "),
    }
}
