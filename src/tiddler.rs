#![allow(dead_code)]

use serde::Deserialize;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
pub struct Tiddler {
    pub title: String,

    #[serde(default)]
    pub text: String,

    #[serde(rename = "tags", default)]
    tag_string: Option<String>,

    #[serde(default)]
    pub created: Option<String>,

    #[serde(default)]
    pub modified: Option<String>,

    #[serde(rename = "type", default)]
    pub tiddler_type: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Tiddler {
    /// TiddlyWiki stores tags as one space-separated string, with multi-word tags
    /// bracketed: "tag1 tag2 [[multi word tag]]". This extracts them into a Vec.
    pub fn tags(&self) -> Vec<String> {
        let mut tags = Vec::new();

        let tag_string = if let Some(tag_string) = &self.tag_string {
            tag_string
        } else {
            &String::new()
        };

        let mut chars = tag_string.chars().peekable();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tags() {
        let tag_string = Some("tag1 tag2 [[multi word tag]]".to_string());

        let tiddler = Tiddler {
            tag_string,
            ..Default::default()
        };

        let v = tiddler.tags();

        let expected = vec!["tag1", "tag2", "multi word tag"];

        assert_eq!(v, expected);
    }
}
