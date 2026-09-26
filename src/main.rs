use std::fs::File;
use std::io::prelude::*;

use anyhow::Result;

// For decoding images
use base64::{Engine as _, engine::general_purpose};

mod tiddler;
use tiddler::Tiddler;

mod markdown;
use markdown::Markdown;

mod naming;

#[allow(unused_imports)]
use log::{debug, error, info, log_enabled};

use crate::{naming::map_name, parser::parse_tiddler};

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

    // if tiddler.title.contains([':', '/', '\\']) {
    //     error!(
    //         "Tiddler [{}] cannot be converted as the title contains either :', '/', or '\\'",
    //         tiddler.title
    //     );
    //     return Ok(());
    // }

    let converted_tiddler_title = map_name(&tiddler.title);

    match tiddler.tiddler_type.as_deref() {
        None => {
            info!("Generating: {} ", tiddler.title);

            let mut input = tiddler.text.as_str();
            let ast = parse_tiddler(&mut input).unwrap();
            let markdown = Markdown::from_inlines(&ast).to_string();

            let file_name = format!("./output/{}.md", converted_tiddler_title);
            let mut file = File::create(file_name).expect("Could not create file!");

            file.write_all(markdown.as_bytes()).expect("write failed");
        }

        Some("text/x-markdown") => {
            info!("Copying jpeg image : {}", tiddler.title);

            let markdown = tiddler.text.as_str();

            let file_name = format!("./output/{}.md", converted_tiddler_title);
            let mut file = File::create(file_name).expect("Could not create file!");

            file.write_all(markdown.as_bytes()).expect("write failed");
        }
        Some("image/jpeg") => {
            info!("Copying jpeg: {}", tiddler.title);
            let base64_as_text = tiddler.text.as_str();
            // Strip whitespace/newlines that may be present in the stored text
            let cleaned: String = base64_as_text
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
            let image_bytes = general_purpose::STANDARD.decode(cleaned)?;

            let file_name = format!("./output/{}", converted_tiddler_title);
            let mut file = File::create(file_name).expect("Could not create file!");

            file.write(&image_bytes)?;
        }
        Some(other) => error!(
            "Cannot handle tiddler type {} for tiddler {}",
            other, tiddler.title
        ),
    }

    // Parse the tiddler text

    Ok(())
}
