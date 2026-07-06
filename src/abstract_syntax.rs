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
        width: usize,
        height: usize,
        link: String,
    },
    Link {
        display_text: Option<String>,
        link: String,
    },
    OrderedList {
        level: usize,
    },
    UnorderedList {
        level: usize,
    },
    TableCell {
        text: String,
        heading: bool,
    },
    Transclusion {
        link: String,
    },
    Table {
        rows: Vec<TableRow>,
    },
}

#[derive(Debug, PartialEq, Clone)]
struct TableRow {
    cells: Vec<TableCell>,
}

#[derive(Debug, PartialEq, Clone)]
struct TableCell {
    text: String,
    vertical_alignment: CellVerticalAlignment,
    horizontal_alignment: CellHorizontalAlignment,
    header: bool,
    merge: CellMerge,
}

#[derive(Debug, PartialEq, Clone)]
enum CellVerticalAlignment {
    Top,
    Bottom,
    Center,
}

#[derive(Debug, PartialEq, Clone)]
enum CellHorizontalAlignment {
    Left,
    Right,
    Center,
}

#[derive(Debug, PartialEq, Clone)]
enum CellMerge {
    Above,
    Left,
    Right,
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
}
