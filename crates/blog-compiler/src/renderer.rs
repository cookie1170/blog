pub const DATE_FORMAT: &str = "%d %B %Y";

pub struct Renderer {
    typst: TypstCompiler,
}

impl Renderer {
    pub fn new() -> Self {
        let typst = TypstCompiler::new();
        Self { typst }
    }
}

struct RendererInner<'i, 'm, 'r, 's, O: io::Write> {
    events: IntoIter<Event<'i>>,
    post_meta: &'m PostRootMeta,
    output: IoWriter<BufWriter<O>>,
    outer: &'r mut Renderer,
    slug: &'s str,
    dev: bool,
}

impl<'i, 'm, 'r, 's, O: io::Write> RendererInner<'i, 'm, 'r, 's, O> {
    pub fn write_beginning_html(&mut self) -> Result<()> {
        self.write_fmt(format_args!(
            r#"
<!doctype html>
<html lang="en-US">
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width" />
        <link href="{PREFIX}/public/style.css" rel="stylesheet" />
        <title>{title} -- Cookie's blog</title>
        <base href="/blog/{slug}/index.html">
        {dev_script}
    </head>
    <body>
        <div class="post">
            <div class="head">
                <h1 class="title">{title}</h1>
                <div class="tags">{tags}</div>
                <p class="date">{date}</h1>
            </div>
            <hr class="section-split" />
            <div class="body">
        "#,
            title = self.post_meta.title,
            date = self.post_meta.date.strftime(DATE_FORMAT),
            slug = { self.slug },
            tags = self.format_tags()?,
            dev_script = if self.dev {
                format!(r#"<script src="{}/dev_public/reload.js"></script>"#, PREFIX)
            } else {
                String::new()
            }
        ))?;
        Ok(())
    }

    pub fn write_ending_html(&mut self) -> Result<()> {
        self.write(
            r#"
            </div>
        </div>
    </body>
</html>
            "#,
        )?;
        Ok(())
    }

    pub fn format_tags(&self) -> Result<String> {
        let mut result = String::with_capacity(64);
        for tag in &self.post_meta.tags {
            write!(&mut result, r#"<div class="tag">{}</div>"#, tag)?;
        }

        Ok(result)
    }

    pub fn process_event(&mut self, event: Event<'i>) -> Result<()> {
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
                let result = self
                    .outer
                    .typst
                    .compile(math.to_string())
                    .with_context(|| {
                        format!("failed to compile typst expression '{}'", math.trim())
                    })?;
                self.write(result)?
            }
            E::DisplayMath(math) => {
                let result = self
                    .outer
                    .typst
                    .compile(math.to_string())
                    .with_context(|| {
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
                    if state {
                        format!(r#"<use href="{PREFIX}/public/check.svg#check"></use>"#)
                    } else {
                        String::new()
                    }
                ))?;
            }
            E::MetadataBlock(meta) => {
                tracing::info!("found meta:\n{meta}");
            }
        }

        Ok(())
    }

    pub fn start_tag(&mut self, tag: Tag) -> Result<()> {
        match tag {
            Tag::Footnote { name, number } => {
                // <label class="footnote" id="{name}-label" for="{name}-input">{number}</label>
                self.write(" <label class=\"footnote\" for=\"")?;
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
            Tag::Paragraph => self.write("<p>")?,
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
                } else if !dest_url.contains("://") {
                    self.write_fmt(format_args!("{PREFIX}/{}", { self.slug }))?;
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
                self.write_fmt(format_args!(r#"<img src="{PREFIX}/{}/"#, { self.slug }))?;
                self.write_escaped_href(&dest_url)?;
                self.write("\" alt=\"")?;
                self.raw_text()?;
                if !title.is_empty() {
                    self.write("\" title=\"")?;
                    self.write_escaped_attr(&title)?;
                }
                self.write("\" />")?;
            }
        }
        Ok(())
    }

    fn end_tag(&mut self, tag_end: TagEnd) -> Result<()> {
        match tag_end {
            TagEnd::List(ordered) => {
                if ordered {
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

impl Renderer {
    pub fn render<O: io::Write>(
        &mut self,
        source: &str,
        slug: &str,
        output: BufWriter<O>,
        dev: bool,
    ) -> Result<PostRootMeta> {
        let ParseResult { root_meta, events } = parser::parse(source)?;
        let events = events.into_iter();
        let post_meta: PostRootMeta =
            toml::from_str(&root_meta).context("invalid root metadata")?;
        let mut renderer = RendererInner {
            events,
            post_meta: &post_meta,
            output: IoWriter(output),
            outer: self,
            slug,
            dev,
        };

        renderer.write_beginning_html()?;
        while let Some(event) = renderer.events.next() {
            renderer.process_event(event)?;
        }
        renderer.write_ending_html()?;
        Ok(post_meta)
    }
}

pub fn write_index(mut writer: impl Write, metas: &[&PostMeta], dev: bool) -> Result<()> {
    let mut posts = String::with_capacity(512 * metas.len());
    for post in metas {
        let mut tags = String::with_capacity(40 * post.tags.len());
        for tag in &post.tags {
            write!(&mut tags, r#"<span class="tag">{tag}</span>"#)?;
        }

        write!(
            &mut posts,
            r#"
                <article class="post-embed">
                    <a class="block-link" href="/blog/{slug}">
                        <h1 class="title on-container">{title}</h1>
                    </a>
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
                        <a class="block-link" href="/blog/${{post.slug}}">
                            <h1 class="title on-container">${{post.title}}</h1>
                        </a>
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
        <div class="post">
            <div class="head">
                <h1 class="title">Cookie's Blog</h1>
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
            </div>
            <hr class="section-split" />
            <div id="search-container"></div>
            <div class="posts-container">
                {posts}
            </div>
        </div>
    </body>
</html>
"##,
        dev_script = if dev {
            format!(r#"<script src="{PREFIX}/dev_public/reload.js"></script>"#)
        } else {
            String::new()
        }
    )?;

    Ok(())
}

use crate::{
    PREFIX, PostMeta,
    parser::{self, Event, ParseResult, PostRootMeta, Tag, TagEnd},
};
use anyhow::{Context, Result};
use pulldown_cmark::{Alignment, BlockQuoteKind, CodeBlockKind, LinkType};
use pulldown_cmark_escape::IoWriter;
use pulldown_cmark_escape::StrWrite;
use std::{
    fmt,
    io::{self, BufWriter, Write},
    vec::IntoIter,
};

use crate::typst::TypstCompiler;
