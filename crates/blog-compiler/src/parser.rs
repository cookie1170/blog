pub struct ParseResult<'i> {
    pub root_meta: CowStr<'i>,
    pub events: Vec<Event<'i>>,
}

#[derive(Debug)]
struct Footnote<'i> {
    number: usize,
    events: Vec<Event<'i>>,
}

struct Parser<'i> {
    events: Vec<Event<'i>>,
    footnotes: HashMap<CowStr<'i>, Footnote<'i>>,
    current_footnote: Option<(CowStr<'i>, Footnote<'i>)>,
    current_alignments: Vec<Alignment>,
    current_metadata_block: String,
    current_cell_index: usize,
    is_table_header: bool,
    in_metadata_block: bool,
    ignore: bool,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct PostRootMeta {
    pub title: String,
    pub date: Date,
    pub tags: Vec<String>,
}

pub fn parse<'i>(source: &'i str) -> Result<ParseResult<'i>> {
    let options = Options::ENABLE_GFM
        | Options::ENABLE_MATH
        | Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_SUBSCRIPT
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_SUPERSCRIPT
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS;
    let mut cmark_parser =
        pulldown_cmark::TextMergeStream::new(CmarkParser::new_ext(source, options));

    let meta;
    if let Some(CE::Start(CTag::MetadataBlock(MetadataBlockKind::PlusesStyle))) =
        cmark_parser.next()
        && let Some(CE::Text(m)) = cmark_parser.next()
        && let Some(CE::End(CTagEnd::MetadataBlock(MetadataBlockKind::PlusesStyle))) =
            cmark_parser.next()
    {
        meta = m;
    } else {
        bail!("expected plus-delimited metadata block at the start");
    }

    let mut events = Vec::with_capacity(1024);
    // HACK: we wrap all the events in `Option` so that we can move out of them
    // in `scan_footnotes` or for `process_events` separately. the set of events will not overlap
    events.extend(cmark_parser.map(Some));

    let mut parser = Parser {
        events: Vec::with_capacity(1024),
        footnotes: HashMap::with_capacity(8),
        current_footnote: None,
        current_alignments: Vec::default(),
        current_cell_index: 0,
        is_table_header: false,
        in_metadata_block: false,
        current_metadata_block: String::new(),
        ignore: false,
    };

    let mut iter = events.iter_mut();
    while let Some(event) = iter.next() {
        parser.scan_footnotes(event)?;
    }

    let mut iter = events.iter_mut();
    while let Some(event) = iter.next() {
        if !parser.is_footnote() && parser.ignore {
            if let Some(CE::End(CTagEnd::FootnoteDefinition)) = event {
                parser.ignore = false;
            } else {
                continue;
            }
        }
        // events that are between `CTag::FootnoteDefinition` and `CTagEnd::FootnoteDefinition`
        // are `take()`n, but the rest are kept, so we're fine to `take()` and unwrap here
        parser.process_event(event.take().unwrap())?;
    }

    Ok(ParseResult {
        root_meta: meta,
        events: parser.events,
    })
}

impl<'i> Parser<'i> {
    fn scan_footnotes(&mut self, event: &mut Option<CE<'i>>) -> Result<()> {
        match event {
            Some(CE::Start(CTag::FootnoteDefinition(name))) => {
                self.current_footnote = Some((
                    name.clone(),
                    Footnote {
                        number: self.footnotes.len() + 1,
                        events: Vec::with_capacity(128),
                    },
                ));
            }
            Some(CE::End(CTagEnd::FootnoteDefinition)) => {
                let (name, footnote) = self
                    .current_footnote
                    .take()
                    .context("footnote definition ended before it started")?;
                self.footnotes.insert(name, footnote);
            }
            other if self.current_footnote.is_some() => {
                self.process_event(other.take().unwrap())?
            }
            _ => {}
        }
        Ok(())
    }

    fn process_event(&mut self, event: CE<'i>) -> Result<()> {
        match event {
            CE::Start(tag) => self.start_tag(tag),
            CE::End(tag_end) => self.end_tag(tag_end),
            CE::Text(cow_str) => {
                if self.in_metadata_block {
                    self.current_metadata_block.push_str(&cow_str);
                } else {
                    self.events().push(Event::Text(cow_str));
                }
            }
            CE::Code(cow_str) => self.events().push(Event::Code(cow_str)),
            CE::InlineMath(cow_str) => self.events().push(Event::InlineMath(cow_str)),
            CE::DisplayMath(cow_str) => self.events().push(Event::DisplayMath(cow_str)),
            CE::Html(cow_str) => self.events().push(Event::Html(cow_str)),
            CE::InlineHtml(cow_str) => self.events().push(Event::InlineHtml(cow_str)),
            CE::FootnoteReference(cow_str) => {
                let footnote = self
                    .footnotes
                    .get(&cow_str)
                    .with_context(|| format!("footnote {cow_str} not found"))?;

                // HACK: due to the lack of view types, we have to copy & paste the `self.events()` method here
                // so we can borrow both `self.footnotes` _and_ `self.events`,
                // whereas `self.events()` would've borrowed all of `self`
                let events = match self.current_footnote {
                    Some((_, ref mut f)) => &mut f.events,
                    None => &mut self.events,
                };

                events.push(Event::Start(Tag::Footnote {
                    name: cow_str,
                    number: footnote.number,
                }));
                events.extend(footnote.events.iter().cloned());
                events.push(Event::End(TagEnd::Footnote));
            }
            CE::SoftBreak => self.events().push(Event::SoftBreak),
            CE::HardBreak => self.events().push(Event::HardBreak),
            CE::Rule => self.events().push(Event::Rule),
            CE::TaskListMarker(checked) => self.events().push(Event::TaskListMarker(checked)),
        }
        Ok(())
    }

    fn start_tag(&mut self, tag: CTag<'i>) {
        match tag {
            CTag::Paragraph => {
                // for footnotes, we replace `<p>`s with `<br />`s
                if !self.is_footnote() {
                    self.events().push(Event::Start(Tag::Paragraph));
                }
            }
            CTag::Heading {
                level,
                id,
                classes,
                attrs,
            } => self.events().push(Event::Start(Tag::Heading {
                level,
                id,
                classes,
                attrs,
            })),
            CTag::BlockQuote(kind) => self.events().push(Event::Start(Tag::BlockQuote(kind))),
            CTag::CodeBlock(kind) => self.events().push(Event::Start(Tag::CodeBlock(kind))),
            CTag::HtmlBlock => self.events().push(Event::Start(Tag::HtmlBlock)),
            CTag::List(first) => self.events().push(Event::Start(Tag::List(first))),
            CTag::Item => self.events().push(Event::Start(Tag::Item)),
            CTag::Table(alignments) => {
                self.current_alignments = alignments;
                self.events().push(Event::Start(Tag::Table));
            }
            CTag::TableHead => {
                self.is_table_header = true;
                self.current_cell_index = 0;
                self.events().push(Event::Start(Tag::TableHead));
            }
            CTag::TableRow => {
                self.current_cell_index = 0;
                self.events().push(Event::Start(Tag::TableRow));
            }
            CTag::TableCell => {
                let alignment = self
                    .current_alignments
                    .get(self.current_cell_index)
                    .copied()
                    .unwrap_or(Alignment::None);
                self.current_cell_index += 1;
                if self.is_table_header {
                    self.events()
                        .push(Event::Start(Tag::TableHeadCell(alignment)));
                } else {
                    self.events().push(Event::Start(Tag::TableCell(alignment)));
                }
            }
            CTag::Emphasis => self.events().push(Event::Start(Tag::Emphasis)),
            CTag::Strong => self.events().push(Event::Start(Tag::Strong)),
            CTag::Strikethrough => self.events().push(Event::Start(Tag::Strikethrough)),
            CTag::Superscript => self.events().push(Event::Start(Tag::Superscript)),
            CTag::Subscript => self.events().push(Event::Start(Tag::Subscript)),
            CTag::Link {
                link_type,
                dest_url,
                title,
                id,
            } => self.events().push(Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            })),
            CTag::Image {
                link_type,
                dest_url,
                title,
                id,
            } => self.events().push(Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            })),
            CTag::MetadataBlock(MetadataBlockKind::PlusesStyle) => self.in_metadata_block = true,

            CTag::FootnoteDefinition(_) => {
                // handled by `scan_footnotes`
                self.ignore = true;
            }
            CTag::MetadataBlock(MetadataBlockKind::YamlStyle) => unimplemented!(),
            CTag::DefinitionList => unimplemented!(),
            CTag::DefinitionListTitle => unimplemented!(),
            CTag::DefinitionListDefinition => unimplemented!(),
        }
    }

    fn end_tag(&mut self, tag: CTagEnd) {
        match tag {
            CTagEnd::Paragraph => {
                if self.is_footnote() {
                    self.events().push(Event::HardBreak);
                } else {
                    self.events().push(Event::End(TagEnd::Paragraph))
                }
            }
            CTagEnd::Heading(heading_level) => self
                .events()
                .push(Event::End(TagEnd::Heading(heading_level))),
            CTagEnd::BlockQuote(block_quote_kind) => self
                .events()
                .push(Event::End(TagEnd::BlockQuote(block_quote_kind))),
            CTagEnd::CodeBlock => self.events().push(Event::End(TagEnd::CodeBlock)),
            CTagEnd::HtmlBlock => self.events().push(Event::End(TagEnd::HtmlBlock)),
            CTagEnd::List(ordered) => self.events().push(Event::End(TagEnd::List(ordered))),
            CTagEnd::Item => self.events().push(Event::End(TagEnd::Item)),

            CTagEnd::Table => self.events().push(Event::End(TagEnd::Table)),
            CTagEnd::TableHead => {
                self.is_table_header = false;
                self.events().push(Event::End(TagEnd::TableHead));
            }
            CTagEnd::TableRow => self.events().push(Event::End(TagEnd::TableRow)),
            CTagEnd::TableCell => {
                if self.is_table_header {
                    self.events().push(Event::End(TagEnd::TableHeadCell));
                } else {
                    self.events().push(Event::End(TagEnd::TableCell))
                }
            }
            CTagEnd::Emphasis => self.events().push(Event::End(TagEnd::Emphasis)),
            CTagEnd::Strong => self.events().push(Event::End(TagEnd::Strong)),
            CTagEnd::Strikethrough => self.events().push(Event::End(TagEnd::Strikethrough)),
            CTagEnd::Superscript => self.events().push(Event::End(TagEnd::Superscript)),
            CTagEnd::Subscript => self.events().push(Event::End(TagEnd::Subscript)),
            CTagEnd::Link => self.events().push(Event::End(TagEnd::Link)),
            CTagEnd::Image => self.events().push(Event::End(TagEnd::Image)),
            CTagEnd::MetadataBlock(MetadataBlockKind::PlusesStyle) => {
                let meta = std::mem::replace(&mut self.current_metadata_block, String::new());
                self.events().push(Event::MetadataBlock(meta));
                self.in_metadata_block = false;
            }

            CTagEnd::FootnoteDefinition => self.ignore = false,
            CTagEnd::MetadataBlock(MetadataBlockKind::YamlStyle) => unimplemented!(),
            CTagEnd::DefinitionList => unimplemented!(),
            CTagEnd::DefinitionListTitle => unimplemented!(),
            CTagEnd::DefinitionListDefinition => unimplemented!(),
        }
    }

    fn events(&mut self) -> &mut Vec<Event<'i>> {
        match self.current_footnote {
            Some((_, ref mut f)) => &mut f.events,
            None => &mut self.events,
        }
    }

    fn is_footnote(&self) -> bool {
        self.current_footnote.is_some()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Event<'a> {
    /// Start of a tagged element. Events that are yielded after this event
    /// and before its corresponding `End` event are inside this element.
    /// Start and end events are guaranteed to be balanced.
    Start(Tag<'a>),
    /// End of a tagged element.
    End(TagEnd),
    /// A text node.
    ///
    /// All text, outside and inside [`Tag`]s.
    Text(CowStr<'a>),
    /// An [inline code node](https://spec.commonmark.org/0.31.2/#code-spans).
    ///
    /// ```markdown
    /// `code`
    /// ```
    Code(CowStr<'a>),
    /// An inline math environment node.
    /// Requires [`Options::ENABLE_MATH`].
    ///
    /// ```markdown
    /// $math$
    /// ```
    InlineMath(CowStr<'a>),
    /// A display math environment node.
    /// Requires [`Options::ENABLE_MATH`].
    ///
    /// ```markdown
    /// $$math$$
    /// ```
    DisplayMath(CowStr<'a>),
    /// An HTML node.
    ///
    /// A line of HTML inside [`Tag::HtmlBlock`] includes the line break.
    Html(CowStr<'a>),
    /// An [inline HTML node](https://spec.commonmark.org/0.31.2/#raw-html).
    ///
    /// Contains only the tag itself, e.g. `<open-tag>`, `</close-tag>` or `<!-- comment -->`.
    ///
    /// **Note**: Under some conditions HTML can also be parsed as an HTML Block, see [`Tag::HtmlBlock`] for details.
    InlineHtml(CowStr<'a>),
    /// A metadata block
    MetadataBlock(String),

    /// A [soft line break](https://spec.commonmark.org/0.31.2/#soft-line-breaks).
    ///
    /// Any line break that isn't a [`HardBreak`](Self::HardBreak), or the end of e.g. a paragraph.
    SoftBreak,
    /// A [hard line break](https://spec.commonmark.org/0.31.2/#hard-line-breaks).
    ///
    /// A line ending that is either preceded by at least two spaces or `\`.
    ///
    /// ```markdown
    /// hard··
    /// line\
    /// breaks
    /// ```
    /// *`·` is a space*
    HardBreak,
    /// A horizontal ruler.
    ///
    /// ```markdown
    /// ***
    /// ···---
    /// _·_··_····_··
    /// ```
    /// *`·` is any whitespace*
    Rule,
    /// A task list marker, rendered as a checkbox in HTML. Contains a true when it is checked.
    /// Only parsed and emitted with [`Options::ENABLE_TASKLISTS`].
    /// ```markdown
    /// - [ ] unchecked
    /// - [x] checked
    /// ```
    TaskListMarker(bool),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Tag<'a> {
    /// A paragraph of text and other inline elements.
    Paragraph,

    /// A heading, with optional identifier, classes and custom attributes.
    /// The identifier is prefixed with `#` and the last one in the attributes
    /// list is chosen, classes are prefixed with `.` and custom attributes
    /// have no prefix and can optionally have a value (`myattr` or `myattr=myvalue`).
    ///
    /// `id`, `classes` and `attrs` are only parsed and populated with [`Options::ENABLE_HEADING_ATTRIBUTES`], `None` or empty otherwise.
    Heading {
        level: HeadingLevel,
        id: Option<CowStr<'a>>,
        classes: Vec<CowStr<'a>>,
        /// The first item of the tuple is the attr and second one the value.
        attrs: Vec<(CowStr<'a>, Option<CowStr<'a>>)>,
    },

    /// A block quote.
    ///
    /// The `BlockQuoteKind` is only parsed & populated with [`Options::ENABLE_GFM`], `None` otherwise.
    ///
    /// ```markdown
    /// > regular quote
    ///
    /// > [!NOTE]
    /// > note quote
    /// ```
    BlockQuote(Option<BlockQuoteKind>),
    /// A code block.
    CodeBlock(CodeBlockKind<'a>),

    /// An HTML block.
    ///
    /// A line that begins with some predefined tags (HTML block tags)
    /// (see [CommonMark Spec](https://spec.commonmark.org/0.31.2/#html-blocks) for more details) or any tag that is followed only by whitespace.
    ///
    /// Most HTML blocks end on an empty line, though some e.g. `<pre>` like `<script>` or `<!-- Comments -->` don't.
    /// ```markdown
    /// <body> Is HTML block even though here is non-whitespace.
    /// Block ends on an empty line.
    ///
    /// <some-random-tag>
    /// This is HTML block.
    ///
    /// <pre> Doesn't end on empty lines.
    ///
    /// This is still the same block.</pre>
    /// ```
    HtmlBlock,

    /// A list. If the list is ordered the field indicates the number of the first item.
    /// Contains only list items.
    List(Option<u64>),
    /// A list item.
    Item,

    /// A table. Contains a vector describing the text-alignment for each of its columns.
    Table,
    /// A table header. Contains only `TableHeadCell`s. Note that the table body starts immediately
    /// after the closure of the `TableHead` tag. There is no `TableBody` tag.
    TableHead,
    TableHeadCell(Alignment),
    /// A table row. Contains only `TableCell`s.
    TableRow,
    TableCell(Alignment),

    // span-level tags
    /// [Emphasis](https://spec.commonmark.org/0.31.2/#emphasis-and-strong-emphasis).
    /// ```markdown
    /// half*emph* _strong_ _multi _level__
    /// ```
    Emphasis,
    /// [Strong emphasis](https://spec.commonmark.org/0.31.2/#emphasis-and-strong-emphasis).
    /// ```markdown
    /// half**strong** __strong__ __multi __level____
    /// ```
    Strong,
    ///
    /// ```markdown
    /// ~strike through~
    /// ```
    Strikethrough,
    ///
    /// ```markdown
    /// ^superscript^
    /// ```
    Superscript,
    /// ```markdown
    /// ~subscript~ ~~if also enabled this is strikethrough~~
    /// ```
    Subscript,

    /// A link.
    Link {
        link_type: LinkType,
        dest_url: CowStr<'a>,
        title: CowStr<'a>,
        /// Identifier of reference links, e.g. `world` in the link `[hello][world]`.
        id: CowStr<'a>,
    },

    /// An image. The first field is the link type, the second the destination URL and the third is a title,
    /// the fourth is the link identifier.
    Image {
        link_type: LinkType,
        dest_url: CowStr<'a>,
        title: CowStr<'a>,
        /// Identifier of reference links, e.g. `world` in the link `[hello][world]`.
        id: CowStr<'a>,
    },

    /// A reference to a footnote, which will be followed by all the events belonging to it
    ///
    /// ```markdown
    /// [^1]
    /// ```
    Footnote {
        name: CowStr<'a>,
        number: usize,
    },
}

#[derive(PartialEq, Debug, Clone)]
pub enum TagEnd {
    Paragraph,
    Heading(HeadingLevel),

    BlockQuote(Option<BlockQuoteKind>),
    CodeBlock,

    HtmlBlock,

    /// A list, `true` for ordered lists.
    List(bool),
    Item,
    Footnote,

    Table,
    TableHead,
    TableRow,
    TableHeadCell,
    TableCell,

    Emphasis,
    Strong,
    Strikethrough,
    Superscript,
    Subscript,

    Link,
    Image,
}
use std::collections::HashMap;

use anyhow::{Context as _, Result, bail};
use jiff::civil::Date;
use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, CowStr, Event as CE, HeadingLevel, LinkType,
    MetadataBlockKind, Options, Parser as CmarkParser, Tag as CTag, TagEnd as CTagEnd,
};
use serde::Deserialize;
