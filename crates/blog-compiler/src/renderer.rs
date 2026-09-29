pub const DATE_FORMAT: &str = "%d %B %Y";

#[derive(PartialEq, Debug, Clone)]
pub struct Renderer {
    typst: TypstCompiler,
    in_path: PathBuf,
    out_path: PathBuf,
}

struct RendererInner<'i, 'm, 'r, O: io::Write> {
    events: Peekable<Iter<'i, Event>>,
    meta: &'m PostMeta,
    metas: &'m [PostMeta],
    output: IoWriter<BufWriter<O>>,
    typst: &'r mut TypstCompiler,
    dev: bool,
}

impl<'i, 'm, 'r, O: io::Write> RendererInner<'i, 'm, 'r, O> {
    pub fn write_beginning_html(&mut self) -> Result<()> {
        let dev = self.dev;
        self.write_fmt(format_args!(
            r#"
<!doctype html>
<html lang="en-US">
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width" />
        <link href="{PREFIX}/public/style.css" rel="stylesheet" />
        <title>{title} -- Cookie's blog</title>
        {dev_script}
    </head>
    <body>
        <main class="post">
            <header class="head">
                <h1 class="title">{title}</h1>
                <div class="tags">{tags}</div>
                <p class="date">{date}</h1>
            </header>
            <hr class="section-split" />
        "#,
            title = self.meta.title,
            date = self.meta.date.strftime(DATE_FORMAT),
            tags = self.format_tags()?,
            dev_script = fmt::from_fn(|f| {
                if dev {
                    write!(
                        f,
                        r#"<script src="{}/dev_public/reload.js"></script>"#,
                        PREFIX
                    )
                } else {
                    Ok(())
                }
            })
        ))?;
        Ok(())
    }

    pub fn write_ending_html(&mut self) -> Result<()> {
        if self.meta.prev.is_some() || self.meta.next.is_some() {
            self.write(
                r#"
        <hr class="section-split" />
                "#,
            )?;
        }

        if let Some(ref prev) = self.meta.prev {
            let Some(prev_post) = self.metas.iter().find(|p| p.slug == *prev) else {
                bail!("previous post '{prev}' not found");
            };

            self.write_fmt(format_args!(
                r#"
        <div class="prev">
            <a class="a" href="{PREFIX}/{prev}">
                <svg height="16px" width="16px">
                    <use href="{PREFIX}/public/left.svg#left">
                </svg>
                {prev_title}
            </a>
        </div>
                "#,
                prev_title = prev_post.title,
            ))?;
        }

        if let Some(ref next) = self.meta.next {
            let Some(next_post) = self.metas.iter().find(|p| p.slug == *next) else {
                bail!("next post '{next}' not found");
            };

            self.write_fmt(format_args!(
                r#"
        <div class="next">
            <a class="a" href="{PREFIX}/{next}">
                 {next_title}
                 <svg height="16px" width="16px">
                     <use href="{PREFIX}/public/right.svg#right">
                 </svg>
            </a>
        </div>
                "#,
                next_title = next_post.title,
            ))?;
        }
        self.write(
            r#"
    </main>
</html>
            "#,
        )?;
        Ok(())
    }

    pub fn format_tags(&self) -> Result<String> {
        let mut result = String::with_capacity(64);
        for tag in &self.meta.tags {
            write!(&mut result, r#"<div class="tag">{}</div>"#, tag)?;
        }

        Ok(result)
    }

    pub fn process_event(&mut self, event: &Event) -> Result<()> {
        use Event as E;
        match event {
            E::Start(tag) => self.start_tag(tag)?,
            E::End(tag_end) => self.end_tag(tag_end)?,
            E::Text(text) => self.write_escaped(&text)?,

            E::Code(code) => {
                self.write(r#"<code class="inline-code">"#)?;
                self.write_escaped(&code)?;
                self.write("</code>")?;
            }
            E::InlineMath(math) => {
                let result = self.typst.compile(math.to_string()).with_context(|| {
                    format!("failed to compile typst expression '{}'", math.trim())
                })?;
                self.write(result)?
            }
            E::DisplayMath(math) => {
                let result = self.typst.compile(math.to_string()).with_context(|| {
                    format!("failed to compile typst expression '{}'", math.trim())
                })?;
                self.write("<p>")?;
                self.write(result)?;
                self.write("</p>")?;
            }
            E::Html(html) | E::InlineHtml(html) => self.write(&html as &str)?,
            E::SoftBreak => self.write("\n")?,
            E::HardBreak => self.write("<br />")?,
            E::Rule => self.write("<hr />")?,
            E::TaskListMarker(state) => {
                self.write_fmt(format_args!(
                    r#"
                    <svg height="16px" width="16px" class="task-marker">{}</svg>
                "#,
                    fmt::from_fn(|f| {
                        if *state {
                            write!(f, r#"<use href="{PREFIX}/public/check.svg#check"></use>"#)
                        } else {
                            Ok(())
                        }
                    })
                ))?;
            }
            E::MetadataBlock(meta) => {
                tracing::info!("found meta:\n{meta}");
            }
        }

        Ok(())
    }

    pub fn start_tag(&mut self, tag: &Tag) -> Result<()> {
        match tag {
            Tag::Footnote { name, number } => {
                // <label class="footnote" id="{name}-label" for="{name}-input">{number}</label>
                self.write("<label class=\"footnote\" for=\"")?;
                self.write_escaped_attr(&name)?;
                self.write_fmt(format_args!("-input\">{number}</label>"))?;

                // this input has `display: none`
                // <input class="footnote", id="{name}-input" type="checkbox"/ >
                self.write("<input class=\"footnote\", id=\"")?;
                self.write_escaped_attr(&name)?;
                self.write("-input\" type=\"checkbox\" />")?;

                self.write("<span class=\"footnote\">")?;

                // <label class="footnote" for="{name}-input">{number}}</label>
                self.write("<label class=\"footnote\" for=\"")?;
                self.write_escaped_attr(&name)?;
                self.write_fmt(format_args!("-input\">{number}</label> "))?;
            }
            Tag::Heading {
                level,
                id,
                classes,
                attrs,
            } => {
                self.write_fmt(format_args!("<{}", level))?;
                if let Some(id) = id {
                    self.write(" id=\"")?;
                    self.write_escaped_attr(&id)?;
                    self.write("\"")?;
                }
                let mut classes = classes.iter();
                if let Some(class) = classes.next() {
                    self.write(" class=\"")?;
                    self.write_escaped_attr(class)?;
                    for class in classes {
                        self.write(" ")?;
                        self.write_escaped_attr(class)?;
                    }
                    self.write("\"")?;
                }
                for (attr, value) in attrs {
                    self.write(" ")?;
                    self.write_escaped_attr(&attr)?;
                    if let Some(val) = value {
                        self.write("=\"")?;
                        self.write_escaped_attr(&val)?;
                        self.write("\"")?;
                    } else {
                        self.write("=\"\"")?;
                    }
                }
                self.write(">")?;
            }
            Tag::BlockQuote(block_quote_kind) => {
                self.write("<blockquote")?;
                if let Some(kind) = block_quote_kind {
                    match kind {
                        BlockQuoteKind::Note => self.write(" class=\"note\"")?,
                        BlockQuoteKind::Tip => self.write(" class=\"tip\"")?,
                        BlockQuoteKind::Important => self.write(" class=\"important\"")?,
                        BlockQuoteKind::Warning => self.write(" class=\"warn\"")?,
                        BlockQuoteKind::Caution => self.write(" class=\"caution\"")?,
                    }
                }
                self.write(">")?;
            }
            Tag::CodeBlock(kind) => match kind {
                // TODO: Syntax highlighting for languages
                CodeBlockKind::Fenced(_info) => self.write("<pre class=\"code-wrapper\"><code>")?,
                CodeBlockKind::Indented => self.write("<pre class=\"code-wrapper\"><code>")?,
            },

            Tag::List(None) => self.write("<ul>")?,
            Tag::List(Some(1)) => self.write("<ol>")?,
            Tag::List(Some(start)) => self.write_fmt(format_args!("<ol start=\"{}\">", start))?,
            Tag::Item => self.write("<li>")?,

            Tag::Table => self.write("<table>")?,
            Tag::TableHead => self.write("<thead>")?,
            Tag::TableRow => self.write("<tr>")?,
            Tag::TableCell(alignment) => {
                let alignment_text = match alignment {
                    Alignment::None => "table-align-none",
                    Alignment::Left => "table-align-left",
                    Alignment::Center => "table-align-centre",
                    Alignment::Right => "table-align-right",
                };
                self.write_fmt(format_args!("<td class=\"{alignment_text}\">"))?;
            }

            Tag::TableHeadCell(alignment) => {
                let alignment_text = match alignment {
                    Alignment::None => "table-align-none",
                    Alignment::Left => "table-align-left",
                    Alignment::Center => "table-align-centre",
                    Alignment::Right => "table-align-right",
                };
                self.write_fmt(format_args!("<th class=\"{alignment_text}\">"))?;
            }
            Tag::HtmlBlock => { /* nop */ }
            Tag::Paragraph => {
                self.write("<p")?;
                if self
                    .events
                    .peek()
                    .is_some_and(|e| matches!(e, Event::Start(Tag::Image { .. })))
                {
                    self.write(r#" class="center""#)?;
                }
                self.write(">")?;
            }
            Tag::Emphasis => self.write("<em>")?,
            Tag::Strong => self.write("<strong>")?,
            Tag::Strikethrough => self.write("<del>")?,
            Tag::Superscript => self.write("<sup>")?,
            Tag::Subscript => self.write("<sub>")?,

            Tag::Link {
                link_type: LinkType::Email,
                dest_url,
                title,
                id: _,
            } => {
                self.write(r#"<a class="a" href="mailto:"#)?;
                self.write_escaped_href(&dest_url)?;
                if !title.is_empty() {
                    self.write("\" title=\"")?;
                    self.write_escaped_attr(&title)?;
                }
                self.write("\">")?;
            }
            Tag::Link {
                link_type: _,
                dest_url,
                title,
                id: _,
            } => {
                self.write(r#"<a class="a" href=""#)?;
                if dest_url.starts_with('/') {
                    self.write(PREFIX)?;
                } else if !dest_url.contains("://") && !dest_url.starts_with('#') {
                    self.write_fmt(format_args!("{PREFIX}/{}", self.meta.slug))?;
                }
                self.write_escaped_href(&dest_url)?;
                if !title.is_empty() {
                    self.write("\" title=\"")?;
                    self.write_escaped_attr(&title)?;
                }
                self.write("\">")?;
            }
            Tag::Image {
                link_type: _,
                dest_url,
                title,
                id: _,
            } => {
                let url_prefix = fmt::from_fn(|f| {
                    if !dest_url.starts_with('/') && !dest_url.contains("://") {
                        write!(f, "{PREFIX}/{}/images/", self.meta.slug)?;
                    }
                    Ok(())
                });

                if dest_url.ends_with(".webm") {
                    self.write_fmt(format_args!("<video autoplay loop muted playinline"))?;
                    self.write_image_meta(title)?;
                    self.write(">")?;
                    self.write_fmt(format_args!(r#"<source src="{url_prefix}"#))?;
                    self.write_escaped_href(dest_url)?;
                    self.write(r#"" type="video/webm">"#)?;
                    self.write("</video>")?;
                } else {
                    self.write_fmt(format_args!(r#"<img src="{url_prefix}"#,))?;
                    self.write_escaped_href(&dest_url)?;
                    self.write("\"")?;
                    self.write_image_meta(title)?;
                    self.write(" />")?;
                }
            }
        }
        Ok(())
    }

    fn write_image_meta(&mut self, title: &str) -> Result<(), Error> {
        self.write(r#" alt=""#)?;
        self.raw_text()?;
        if !title.is_empty() {
            self.write(r#"" title=""#)?;
            self.write_escaped_attr(&title)?;
        }
        self.write("\"")?;

        Ok(())
    }

    fn end_tag(&mut self, tag_end: &TagEnd) -> Result<()> {
        match tag_end {
            TagEnd::List(ordered) => {
                if *ordered {
                    self.write("</ol>")?;
                } else {
                    self.write("</ul>")?;
                }
            }
            TagEnd::Heading(level) => self.write_fmt(format_args!("</{level}>"))?,
            TagEnd::Item => self.write("</li>")?,

            TagEnd::Table => self.write("</table>")?,
            TagEnd::TableHead => self.write("</thead>")?,
            TagEnd::TableRow => self.write("</tr>")?,
            TagEnd::TableHeadCell => self.write("</th>")?,
            TagEnd::TableCell => self.write("</td>")?,

            TagEnd::HtmlBlock => { /* nop */ }
            TagEnd::BlockQuote(_) => self.write("</blockquote>")?,
            TagEnd::CodeBlock => self.write("</code></pre>")?,
            TagEnd::Paragraph => self.write("</p>")?,
            TagEnd::Emphasis => self.write("</em>")?,
            TagEnd::Strong => self.write("</strong>")?,
            TagEnd::Strikethrough => self.write("</del>")?,
            TagEnd::Superscript => self.write("</sup>")?,
            TagEnd::Subscript => self.write("</sub>")?,
            TagEnd::Link => self.write("</a>")?,
            TagEnd::Image => self.write("</image>")?,
            TagEnd::Footnote => self.write("</span>")?,
        }

        Ok(())
    }

    fn raw_text(&mut self) -> Result<()> {
        let mut nest = 0;
        while let Some(event) = self.events.next() {
            match event {
                Event::Start(_) => nest += 1,
                Event::End(_) => {
                    if nest == 0 {
                        break;
                    }
                    nest -= 1;
                }
                Event::Html(text)
                | Event::InlineHtml(text)
                | Event::Code(text)
                | Event::Text(text) => {
                    self.write_escaped_attr(&text)?;
                }
                Event::InlineMath(text) => {
                    self.write("$")?;
                    self.write_escaped_attr(&text)?;
                    self.write("$")?;
                }
                Event::DisplayMath(text) => {
                    self.write("$$")?;
                    self.write_escaped_attr(&text)?;
                    self.write("$$")?;
                }
                Event::SoftBreak | Event::HardBreak | Event::Rule => {
                    self.write(" ")?;
                }
                Event::TaskListMarker(true) => self.write("[x]")?,
                Event::TaskListMarker(false) => self.write("[ ]")?,
                Event::MetadataBlock(meta) => self.write_fmt(format_args!("+++\n{meta}\n+++"))?,
            }
        }
        Ok(())
    }

    pub fn write(&mut self, text: impl AsRef<[u8]>) -> Result<()> {
        self.output
            .0
            .write_all(text.as_ref())
            .context("failed to write to output")?;
        Ok(())
    }

    pub fn write_fmt(&mut self, args: fmt::Arguments) -> Result<()> {
        self.output.0.write_fmt(args)?;
        Ok(())
    }

    pub fn write_escaped(&mut self, text: &str) -> Result<()> {
        pulldown_cmark_escape::escape_html_body_text(&mut self.output, text)
            .context("failed to write to output")?;
        Ok(())
    }

    pub fn write_escaped_attr(&mut self, text: &str) -> Result<()> {
        pulldown_cmark_escape::escape_html(&mut self.output, text)
            .context("failed to write to output")?;
        Ok(())
    }

    pub fn write_escaped_href(&mut self, text: &str) -> Result<()> {
        pulldown_cmark_escape::escape_href(&mut self.output, text)
            .context("failed to write to output")?;
        Ok(())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct RenderInput {
    pub meta: PostMeta,
    pub events: Vec<Event>,
    pub metas: Vec<PostMeta>,
    pub dev: bool,
}

impl Renderer {
    pub fn new(in_path: PathBuf, out_path: PathBuf) -> Self {
        let typst = TypstCompiler::new();
        Self {
            typst,
            in_path,
            out_path,
        }
    }
}

impl Process for Renderer {
    type Input = RenderInput;

    fn execute(&mut self, input: &Self::Input) -> Result<Self::Output> {
        info!("compiling post '{}'", input.meta.slug);

        let RenderInput {
            meta,
            events,
            metas,
            dev,
        } = input;
        let events = events.iter().peekable();
        let html_path = self.out_path.join("index.html");
        let output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&html_path)
            .with_context(|| format!("failed to open '{}'", html_path.display()))?;

        let output = BufWriter::new(output);

        let mut renderer = RendererInner {
            events,
            meta: &meta,
            metas: &metas,
            output: IoWriter(output),
            typst: &mut self.typst,
            dev: *dev,
        };

        renderer.write_beginning_html()?;
        while let Some(event) = renderer.events.next() {
            renderer.process_event(event)?;
        }
        renderer.write_ending_html()?;
        Ok(())
    }

    paths! {}
}

pub fn write_index(mut writer: impl Write, metas: &[PostMeta], dev: bool) -> Result<()> {
    let mut posts = String::with_capacity(512 * metas.len());
    for post in metas {
        let mut tags = String::with_capacity(40 * post.tags.len());
        for tag in &post.tags {
            write!(&mut tags, r#"<span class="tag">{tag}</span>"#)?;
        }

        // prerender the posts so they 1. work without javascript, and 2. don't cause lag on startup from `fetch`ing them
        write!(
            &mut posts,
            r#"
                <article class="post-embed">
                    <header>
                        <a class="block-link" href="/blog/{slug}">
                            <h1 class="title on-container">{title}</h1>
                        </a>
                    </header>
                    <div class="tags">{tags}</div>
                    <p class="date">{date}</p>
                </article>
        "#,
            slug = post.slug,
            title = post.title,
            date = post.date.strftime(DATE_FORMAT)
        )?;
    }

    write!(
        &mut writer,
        r##"
<!doctype html>

<html lang="en-US">
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width" />
        <link href="{PREFIX}/public/style.css" rel="stylesheet" />
        <title>Cookie's blog</title>
        {dev_script}
        <script type="module">
            document.querySelector("#search-container").innerHTML = `
                <div class="search-bar">
                    <input id="search" type="text" />
                </div>
            `;
            let posts = await (await fetch("/blog/posts.json")).json();
            document.querySelector("#search").addEventListener("input", search);
            search();

            function rebuildPosts(posts) {{
                let postsContainer = document.querySelector(".posts-container");
                postsContainer.innerHTML = "";
                for (let post of posts) {{
                    let tagsString = "";
                    for (let tag of post.tags) {{
                        tagsString += `<span class="tag">${{tag}}</span>`;
                    }}

                    postsContainer.innerHTML += `
                        <article class="post-embed">
                            <header>
                                <a class="block-link" href="/blog/${{post.slug}}">
                                    <h1 class="title on-container">${{post.title}}</h1>
                                </a>
                            </header>
                            <div class="tags">${{tagsString}}</div>
                            <p class="date">${{post.formatted_date}}</p>
                        </article>
                    `;
                }}
            }}

            function search() {{
                let searchBar = document.querySelector("#search");
                let query = searchBar.value.toLowerCase();
                if (query == "") {{
                    rebuildPosts(posts);
                    return;
                }}

                let splitQuery = query.split(" ");
                let matchingPosts = posts.filter((post) => {{
                    return splitQuery.find(
                        (word) =>
                            post.title.toLowerCase().includes(word) ||
                            post.tags.find((tag) =>
                                tag.toLowerCase().includes(word),
                            ),
                    );
                }});
                rebuildPosts(matchingPosts);
            }}
        </script>
    </head>
    <body>
        <main class="post">
            <header class="head">
                <h1 class="title">Cookie's blog</h1>
                <div class="tags">
                    <span class="tag">Gamedev</span>
                    <span class="tag">Programming</span>
                    <span class="tag">Tutorials</span>
                    <span class="tag">Devlogs</span>
                </div>
                <p>Hi, I'm Cookie and this is my blog!</p>
                <p>
                    I mainly write educational content about programming,
                    graphics programming, and gamedev related topics, and
                    occasional devlogs or random side-tangents. Hope you enjoy
                    it!
                </p>
            </header>
            <hr class="section-split" />
            <div id="search-container"></div>
            <section class="posts-container">
                {posts}
            </section>
        </main>
    </body>
</html>
"##,
        dev_script = fmt::from_fn(|f| {
            if dev {
                write!(
                    f,
                    r#"<script src="{PREFIX}/dev_public/reload.js"></script>"#
                )
            } else {
                Ok(())
            }
        })
    )?;

    Ok(())
}

use crate::{
    PREFIX, PostMeta,
    parser::{CodeBlockKind, Event, Tag, TagEnd},
    paths,
    processor::Process,
};
use anyhow::{Context, Error, Result, bail};
use pulldown_cmark::{Alignment, BlockQuoteKind, LinkType};
use pulldown_cmark_escape::IoWriter;
use pulldown_cmark_escape::StrWrite;
use std::{
    fmt,
    fs::OpenOptions,
    io::{self, BufWriter, Write},
    iter::Peekable,
    path::PathBuf,
    slice::Iter,
};
use tracing::info;

use crate::typst::TypstCompiler;
