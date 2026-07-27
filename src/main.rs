use std::fs::{File, OpenOptions};
use std::io::prelude::*;

use anyhow::{Context, Result};

use log::{Level, debug, error, info, log_enabled};
mod tiddler;
use tiddler::Tiddler;

mod parser;
use parser::parse_wiki_text;

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
        create_tiddler_file(&tiddler).unwrap();
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

    // let mut file = OpenOptions::new()
    //     .append(true)
    //     .open(file_name)
    //     .expect("cannot open file");
    let mut file = File::create(file_name).expect("Could not crate file");

    file.write_all(tiddler.text.as_bytes())
        .expect("write failed");

    Ok(())
}
