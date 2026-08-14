#![allow(dead_code)]

#[derive(Debug, PartialEq, Clone)]
pub enum Inline {
    PlainText {
        text: String,
    },
    Italics {
        text: String,
    },
    Bold {
        text: String,
    },
    Underlined {
        text: String,
    },
    Superscript {
        text: String,
    },
    Subscript {
        text: String,
    },
    Strikethrough {
        text: String,
    },
    Code {
        text: String,
    },
    Highlight {
        text: String,
    },
    BlockQuote {
        text: String,
        citation: Option<String>,
    },
    CodeBlock {
        text: String,
        language: Option<String>,
    },
    Heading {
        text: String,
        level: usize,
    },
    Image {
        width: Option<String>,
        height: Option<String>,
        caption: Option<String>,
        link: String,
    },
    Link {
        display_text: Option<String>,
        link: String,
        external: bool,
    },
    OrderedList {
        //text: String,
        level: usize,
        numbering: Vec<usize>,
    },
    UnorderedList {
        text: String,
        level: usize,
    },
    Transclusion {
        tiddler: String,
    },

    Table {
        rows: Vec<TableRow>,
    },
    EndOfText,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct TableCell {
    pub contents: Vec<Inline>,
    pub alignment: CellAlignment,
    pub header: bool,
    pub merge: CellMerge,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct CellAlignment {
    pub vertical: CellVerticalAlignment,
    pub horizontal: CellHorizontalAlignment,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum CellVerticalAlignment {
    Top,
    Bottom,
    #[default]
    Middle,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum CellHorizontalAlignment {
    Left,
    Right,
    #[default]
    Center,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum CellMerge {
    Above,
    Left,
    Right,
    #[default]
    None,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DimensionField {
    Width(String),
    Height(String),
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Dimensions {
    pub width: Option<String>,
    pub height: Option<String>,
}

impl Inline {
    pub fn plaintext(text: &str) -> Inline {
        Inline::PlainText {
            text: text.to_string(),
        }
    }

    pub fn bold(text: &str) -> Inline {
        Inline::Bold {
            text: text.to_string(),
        }
    }

    pub fn italics(text: &str) -> Inline {
        Inline::Italics {
            text: text.to_string(),
        }
    }

    pub fn underlined(text: &str) -> Inline {
        Inline::Underlined {
            text: text.to_string(),
        }
    }

    pub fn superscript(text: &str) -> Inline {
        Inline::Superscript {
            text: text.to_string(),
        }
    }

    pub fn subscript(text: &str) -> Inline {
        Inline::Subscript {
            text: text.to_string(),
        }
    }

    pub fn strikethrough(text: &str) -> Inline {
        Inline::Strikethrough {
            text: text.to_string(),
        }
    }

    pub fn code(text: &str) -> Inline {
        Inline::Code {
            text: text.to_string(),
        }
    }

    pub fn highlight(text: &str) -> Inline {
        Inline::Highlight {
            text: text.to_string(),
        }
    }

    pub fn blockquote(text: &str, citation: &str) -> Inline {
        let citation_option = (!citation.is_empty()).then(|| citation.to_string());

        Inline::BlockQuote {
            text: text.to_string(),
            citation: citation_option,
        }
    }

    pub fn codeblock(text: &str, language: &str) -> Inline {
        let language_option = (!language.is_empty()).then(|| language.to_string());

        Inline::CodeBlock {
            text: text.to_string(),
            language: language_option,
        }
    }

    pub fn heading(text: &str, level: usize) -> Inline {
        Inline::Heading {
            text: text.to_string(),
            level,
        }
    }

    pub fn image(link: &str, caption: &str, width: &str, height: &str) -> Inline {
        let caption = (!caption.is_empty()).then(|| caption.to_string());
        let width = (!width.is_empty()).then(|| width.to_string());
        let height = (!height.is_empty()).then(|| height.to_string());

        Inline::Image {
            width,
            height,
            caption,
            link: link.to_string(),
        }
    }

    pub fn link(link: &str, display_text: &str) -> Inline {
        let display_text = (!display_text.is_empty()).then(|| display_text.to_string());
        let link = link.to_string();

        Inline::Link {
            display_text,
            link,
            external: false,
        }
    }

    pub fn ext_link(link: &str, display_text: &str) -> Inline {
        let display_text = (!display_text.is_empty()).then(|| display_text.to_string());
        let link = link.to_string();

        Inline::Link {
            display_text,
            link,
            external: true,
        }
    }

    pub fn ordered_list(level: usize, numbering: &str) -> Inline {
        let numbering: Vec<usize> = if !numbering.is_empty() {
            numbering.split('.').map(|n| n.parse().unwrap()).collect()
        } else {
            Vec::new()
        };

        Inline::OrderedList { level, numbering }
    }

    pub fn unordered_list(text: &str, level: usize) -> Inline {
        Inline::UnorderedList {
            text: text.to_string(),
            level,
        }
    }

    pub fn transclusion(tiddler: &str) -> Inline {
        Inline::Transclusion {
            tiddler: tiddler.to_string(),
        }
    }
}

// pub struct TableCell {
//     pub inlines: Vec<Inline>,
//     pub alignment: CellAlignment,
//     pub header: bool,
//     pub merge: CellMerge,
// }
impl TableCell {
    /// Helper function to create a simple table cell
    pub fn new(text: &str) -> TableCell {
        TableCell {
            contents: vec![Inline::plaintext(text)],
            ..Default::default()
        }
    }

    /// Helper function to crate a table cell with a set of inlines
    pub fn new_with_inlines(inlines: Vec<Inline>) -> TableCell {
        TableCell {
            contents: inlines,
            ..Default::default()
        }
    }

    pub fn header(&mut self) -> &mut TableCell {
        self.header = true;
        self
    }

    pub fn top(&mut self) -> &mut TableCell {
        self.alignment.vertical = CellVerticalAlignment::Top;
        self
    }
    pub fn middle(&mut self) -> &mut TableCell {
        self.alignment.vertical = CellVerticalAlignment::Middle;
        self
    }
    pub fn bottom(&mut self) -> &mut TableCell {
        self.alignment.vertical = CellVerticalAlignment::Bottom;
        self
    }

    pub fn left(&mut self) -> &mut TableCell {
        self.alignment.horizontal = CellHorizontalAlignment::Left;
        self
    }

    pub fn center(&mut self) -> &mut TableCell {
        self.alignment.horizontal = CellHorizontalAlignment::Center;
        self
    }
    pub fn right(&mut self) -> &mut TableCell {
        self.alignment.horizontal = CellHorizontalAlignment::Right;
        self
    }

    pub fn build(&mut self) -> TableCell {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_cell() {
        let t = TableCell::new("aaa");

        assert_eq!(
            t,
            TableCell {
                contents: vec![Inline::PlainText {
                    text: String::from("aaa"),
                }],
                ..Default::default()
            }
        );

        let mut t = TableCell::new("bbb").header().build();
        assert_eq!(t.header, true);
        assert_eq!(
            t,
            TableCell {
                contents: vec![Inline::PlainText {
                    text: String::from("bbb"),
                }],
                header: true,
                ..Default::default()
            }
        );

        let t = t.top().build();
        assert_eq!(t.alignment.vertical, CellVerticalAlignment::Top);
    }

    #[test]
    fn test_ordered_list_helper() {
        let inline = Inline::ordered_list(1, "");
        assert_eq!(
            inline,
            Inline::OrderedList {
                level: 1,
                numbering: vec![]
            }
        );

        let inline = Inline::ordered_list(1, "2");
        assert_eq!(
            inline,
            Inline::OrderedList {
                level: 1,
                numbering: vec![2]
            }
        );

        let inline = Inline::ordered_list(2, "2.2");
        assert_eq!(
            inline,
            Inline::OrderedList {
                level: 2,
                numbering: vec![2, 2]
            }
        );

        let inline = Inline::ordered_list(3, "1.2.3");
        assert_eq!(
            inline,
            Inline::OrderedList {
                level: 3,
                numbering: vec![1, 2, 3]
            }
        );
    }
}
