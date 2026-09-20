pub struct TypstCompiler {
    world: TypstWorld,
}

impl TypstCompiler {
    pub fn new() -> Self {
        Self {
            world: TypstWorld::new(String::new()),
        }
    }

    pub fn compile(&mut self, math_expr: String) -> Result<String> {
        self.world.text = format!(
            r#"#set page(width: auto, height: auto, margin: 0cm); #show math.equation: set text(font: "Fira Math"); $ {math_expr} $"#
        );

        let output = typst::compile::<PagedDocument>(&self.world);
        for warning in output.warnings {
            let mut message = warning.message;
            for hint in warning.hints {
                write!(&mut message, "\nhint: {}", hint.v)?;
            }
            let message = message.trim();
            match warning.severity {
                Severity::Error => error!(
                    "error when compiling typst expression {}: {}",
                    math_expr, message
                ),
                Severity::Warning => warn!(
                    "warning when compiling typst expression {}: {}",
                    math_expr, message
                ),
            }
        }

        let output = match output.output {
            Ok(output) => output,
            Err(errors) => {
                let mut message = String::new();
                for error in errors {
                    write!(&mut message, "error: {}", error.message)?;
                    for hint in error.hints {
                        write!(&mut message, "\nhint: {}", hint.v)?;
                    }
                    message.push_str("\n\n");
                }

                bail!(message)
            }
        };

        Ok(typst_svg::svg_merged(
            &output,
            &typst_svg::SvgOptions {
                render_bleed: false,
                pretty: false,
            },
            Abs::zero(),
        ))
    }
}

const FIRA_MATH: &[u8] = include_bytes!("FiraMath-Regular.otf");

struct TypstWorld {
    text: String,
    library: LazyHash<Library>,
    main_file_id: FileId,
    font_store: FontStore,
}

impl TypstWorld {
    fn new(text: String) -> Self {
        let library = LazyHash::new(Library::default());
        let main_file_id = FileId::unique(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("root.typ").unwrap(),
        ));
        let fonts = Font::iter(Bytes::new(FIRA_MATH));
        let mut font_store = FontStore::new();
        font_store.extend(fonts.map(|f| {
            let info = f.info().clone();
            (f, info)
        }));
        Self {
            text,
            library,
            main_file_id,
            font_store,
        }
    }
}

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.font_store.book()
    }

    fn main(&self) -> FileId {
        self.main_file_id.clone()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id != self.main_file_id {
            return Err(FileError::AccessDenied);
        }

        Ok(Source::new(id, self.text.clone()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id != self.main_file_id {
            return Err(FileError::AccessDenied);
        }

        Ok(Bytes::from_string(self.text.clone()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.font_store.font(index)
    }

    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}

use std::fmt::Write as _;

use anyhow::{Result, bail};
use tracing::{error, warn};
use typst::{
    Library, LibraryExt, World,
    diag::{FileError, FileResult, Severity},
    foundations::{Bytes, Datetime, Duration},
    layout::Abs,
    syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_kit::fonts::FontStore;
use typst_layout::PagedDocument;
