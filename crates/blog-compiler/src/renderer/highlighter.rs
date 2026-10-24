/// Highlight a source string
///
/// Returns HTML source highlighted using `<span>`s
pub fn highlight(language: Language, source: &str) -> Result<String> {
    let highlight_names = [
        "keyword",
        "type",
        "type.builtin",
        "property",
        "constant",
        "constant.builtin",
        "constant.numeric",
        "constructor",
        "function",
        "comment",
        "punctuation",
        "label",
        "string",
        "attribute",
        "operator",
    ];
    let treesitter_lang = match language {
        Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        Language::Wesl => tree_sitter_wesl::LANGUAGE.into(),
    };
    let highlights_query = match language {
        Language::Rust => tree_sitter_rust::HIGHLIGHTS_QUERY,
        Language::Wesl => tree_sitter_wesl::HIGHLIGHTS_QUERY,
    };
    let injections_query = match language {
        Language::Rust => tree_sitter_rust::INJECTIONS_QUERY,
        Language::Wesl => tree_sitter_wesl::INJECTIONS_QUERY,
    };
    let locals_query = match language {
        Language::Rust => "",
        Language::Wesl => tree_sitter_wesl::LOCALS_QUERY,
    };
    let mut config = HighlightConfiguration::new(
        treesitter_lang,
        language.to_string(),
        highlights_query,
        injections_query,
        locals_query,
    )?;
    config.configure(&highlight_names);
    let mut highlighter = Highlighter::new();
    let highlighter = highlighter.highlight(&config, source.as_bytes(), None, None, |_| None)?;
    let mut html_renderer = HtmlRenderer::new();

    html_renderer.render(
        highlighter,
        source.as_bytes(),
        &|Highlight(highlight), out| {
            let mut name = highlight_names[highlight];
            // weird naming
            if name == "constant.builtin" || name == "constant.numeric" {
                name = "literal";
            }
            let name = name.replace('.', "-");
            let _ = write!(out, r#"class="{name}""#);
        },
    )?;

    let out = String::from_utf8_lossy_owned(html_renderer.html);
    Ok(out)
}

/// Languages for which syntax highlighting is supported
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Language {
    Rust,
    Wesl,
}

impl FromStr for Language {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "rs" => Self::Rust,
            "wesl" => Self::Wesl,
            other => bail!("unsupported language: '{other}'"),
        })
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Rust => write!(f, "rust")?,
            Language::Wesl => write!(f, "wesl")?,
        }
        Ok(())
    }
}

use std::{fmt, io::Write, str::FromStr};

use anyhow::{Result, bail};
use tree_sitter_highlight::{Highlight, HighlightConfiguration, Highlighter, HtmlRenderer};
