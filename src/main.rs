use std::fs::File;
use std::io::prelude::*;

use anyhow::Result;

mod tiddler;
use tiddler::Tiddler;

mod markdown;
use markdown::Markdown;

#[allow(unused_imports)]
use log::{debug, error, info, log_enabled};

use crate::parser::parse_tiddler;

mod parser;

mod abstract_syntax;

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    info!("TiddlyWiki to Obsidian");

    //let mut tw_file = File::open("./tiddlywiki/empty.html").unwrap();
    let mut tw_file = File::open("/home/andrew/Downloads/SoftwareTools.html").unwrap();
    let mut tw_string = String::new();
    tw_file.read_to_string(&mut tw_string).unwrap();

    let json = extract_tiddler_json(&tw_string);

    //println!("{}", json.unwrap());

    let tiddlers: Vec<Tiddler> = serde_json::from_str(json.unwrap()).unwrap();

    let filtered_tiddlers: Vec<&Tiddler> = tiddlers
        .iter()
        .filter(|t| !t.title.starts_with("$:/"))
        .collect();

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

    if tiddler.title.contains([':', '/', '\\']) {
        error!(
            "Tiddler [{}] cannot be converted as the title contains either :', '/', or '\\'",
            tiddler.title
        );
        return Ok(());
    }

    if tiddler.title == "Coming from Bash" {
        println!("COMING FROM Bash");
    };

    info!("Generating: {}", tiddler.title);
    let file_name = format!("./output/{}.md", tiddler.title);

    // Parse the tiddler text
    let mut input = tiddler.text.as_str();
    let ast = parse_tiddler(&mut input).unwrap();
    let markdown = Markdown::from_inlines(&ast).to_string();

    // let mut file = OpenOptions::new()
    //     .append(true)
    //     .open(file_name)
    //     .expect("cannot open file");
    let mut file = File::create(file_name).expect("Could not create file!");

    file.write_all(markdown.as_bytes()).expect("write failed");

    Ok(())
}
