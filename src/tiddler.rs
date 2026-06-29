use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Tiddler {
    pub title: String,

    #[serde(default)]
    pub text: String,

    #[serde(default)]
    pub tags: Option<String>,

    #[serde(default)]
    pub created: Option<String>,

    #[serde(default)]
    pub modified: Option<String>,

    #[serde(rename = "type", default)]
    pub tiddler_type: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// TiddlyWiki stores tags as one space-separated string, with multi-word tags
/// bracketed: "tag1 tag2 [[multi word tag]]". This extracts them into a Vec.
fn parse_tags(raw: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut chars = raw.chars().peekable();
    let mut current = String::new();

    while let Some(c) = chars.next() {
        if c == '[' && chars.peek() == Some(&'[') {
            chars.next(); // consume second '['
            let mut bracketed = String::new();
            while let Some(&c2) = chars.peek() {
                if c2 == ']' {
                    chars.next();
                    if chars.peek() == Some(&']') {
                        chars.next();
                        break;
                    }
                } else {
                    bracketed.push(c2);
                    chars.next();
                }
            }
            tags.push(bracketed);
        } else if c.is_whitespace() {
            if !current.is_empty() {
                tags.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        tags.push(current);
    }
    tags
}
