use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use anyhow::Result;

use clap::Parser;

// For decoding images
use base64::{Engine as _, engine::general_purpose};

mod tiddler;
use tiddler::Tiddler;

mod markdown;
use markdown::Markdown;

mod naming;

#[allow(unused_imports)]
use log::{debug, error, info, log_enabled};

mod parser;

mod abstract_syntax;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the TiddlyWiki to read and convert.
    in_path: std::path::PathBuf,

    /// The path where the obsidian files are  generated.
    #[arg(short = 'o', long = "output")]
    out_path: std::path::PathBuf,

    /// Name of the tiddler to be converted. If not speicifed all of the tiddlers
    /// are converted.
    #[arg(short = 't', long = "tiddler")]
    tiddler_name: Option<String>,
}

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    info!("TiddlyWiki to Obsidian");

    let args = Cli::parse();

    //let mut tw_file = File::open("./tiddlywiki/empty.html").unwrap();
    //let mut tw_file = File::open("/home/andrew/Downloads/SoftwareTools.html").unwrap();

    let mut tw_file = File::open(&args.in_path).unwrap();
    let mut tw_string = String::new();
    tw_file.read_to_string(&mut tw_string).unwrap();

    let json = extract_tiddler_json(&tw_string);

    let tiddlers: Vec<Tiddler> = serde_json::from_str(json.unwrap()).unwrap();

    let filtered_tiddlers: Vec<&Tiddler> = tiddlers
        .iter()
        .filter(|t| !t.title.starts_with("$:/"))
        .filter(|t| {
            args.tiddler_name
                .as_deref()
                .map_or(true, |name| t.title == name)
        })
        .collect();

    // Create the output directory
    std::fs::create_dir_all(&args.out_path).expect(&format!(
        "Cannot create directory {}",
        args.out_path.display()
    ));

    for tiddler in filtered_tiddlers {
        // As the parser can sometimes panic (rather than returning a error) need to catch this so that
        // other tiddlers can still be processed.
        let result = std::panic::catch_unwind(|| create_tiddler_file(tiddler, &args.out_path));

        match result {
            Ok(Ok(())) => (),
            Ok(Err(e)) => error!("Cannot convert tiddler {}: Error {:?}", tiddler.title, e),
            Err(_) => error!(
                "{}: Parser panicked (likely zero-consumption repeat)",
                tiddler.title
            ),
        }
    }
}

fn extract_tiddler_json(html: &str) -> Option<&str> {
    let marker = r#"class="tiddlywiki-tiddler-store""#;
    let start_idx = html.find(marker)?;
    let tag_close = html[start_idx..].find('>')? + start_idx + 1;
    let end_idx = html[tag_close..].find("</script>")? + tag_close;
    Some(html[tag_close..end_idx].trim())
}

fn create_tiddler_file(tiddler: &Tiddler, out_dir: &Path) -> Result<()> {
    let converted_tiddler_title = naming::map_name(&tiddler.title);

    match tiddler.tiddler_type.as_deref() {
        None | Some("") | Some("text/vnd.tiddlywiki") => {
            info!("Generating: {} ", tiddler.title);

            let mut input = tiddler.text.as_str();
            let ast = parser::parse(&mut input)?;

            let markdown = Markdown::from_inlines(&ast).to_string();

            let path = out_dir.join(format!("{}.md", converted_tiddler_title));
            let mut file = File::create(path)?;

            file.write_all(markdown.as_bytes())?;
        }

        Some("text/x-markdown") => {
            info!("Copying markdown : {}", tiddler.title);

            let markdown = tiddler.text.as_str();

            let path = out_dir.join(format!("{}.md", converted_tiddler_title));
            let mut file = File::create(path)?;

            file.write_all(markdown.as_bytes())?;
        }
        Some(mime @ ("image/jpeg" | "image/png" | "image/gif")) => {
            info!("Copying {}: {}", mime, tiddler.title);

            let image_bytes = convert_image(&tiddler.text)?;

            // let (_, extension) = mime
            //     .split_once('/')
            //     .ok_or_else(|| anyhow::anyhow!("malformed MIMI type: {mime}"))?;

            //let path = out_dir.join(format!("{}.{}", converted_tiddler_title, extension));
            let path = out_dir.join(converted_tiddler_title);
            let mut file = File::create(path)?;

            file.write_all(&image_bytes)?;
        }
        Some("image/svg+xml") => {
            info!("Copying SVG : {}", tiddler.title);

            let markdown = tiddler.text.as_str();

            let path = out_dir.join(format!("{}", converted_tiddler_title));

            let mut file = File::create(path)?;

            file.write_all(markdown.as_bytes())?;
        }

        Some(other) => error!(
            "Cannot handle tiddler type {} for tiddler {}",
            other, tiddler.title
        ),
    }

    // Parse the tiddler text

    Ok(())
}

fn convert_image(base64_as_text: &str) -> Result<Vec<u8>> {
    // Strip whitespace/newlines that may be present in the stored text
    let cleaned: String = base64_as_text
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let image_bytes = general_purpose::STANDARD.decode(cleaned)?;

    Ok(image_bytes)
}
