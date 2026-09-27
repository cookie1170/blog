#![feature(associated_type_defaults)]

mod parser;
pub mod processor;
mod renderer;
mod typst;

pub const PREFIX: &str = "/blog";

#[derive(PartialEq, Debug, Clone)]
pub struct Blog {
    pub in_path: PathBuf,
    pub out_path: PathBuf,
    copy_public_dir: Processor<CopyDir>,
    copy_dev_public_dir: Processor<CopyDir>,
    posts: Vec<Post>,
    posts_path: PathBuf,
}

impl Blog {
    pub fn new(root: PathBuf) -> Self {
        let out_path = root.join("dist");
        Self {
            copy_public_dir: Processor::new(CopyDir {
                in_path: root.join("public"),
                out_path: out_path.join("public"),
            }),
            copy_dev_public_dir: Processor::new(CopyDir {
                in_path: root.join("dev_public"),
                out_path: out_path.join("dev_public"),
            }),
            posts_path: root.join("posts"),
            in_path: root,
            out_path,
            posts: Vec::new(),
        }
    }

    pub fn update_posts(&mut self) -> anyhow::Result<()> {
        self.posts
            .retain(|p| fs::exists(self.posts_path.join(&p.slug)).is_ok_and(|b| b));

        for post in fs::read_dir(&self.posts_path)
            .with_context(|| format!("failed to read {}", self.posts_path.display()))?
        {
            let post = post.context("failed to read post")?;
            if !post
                .metadata()
                .context("failed to get post metadata")?
                .is_dir()
            {
                continue;
            }
            let path = post.path();
            let slug = path
                .file_name()
                .map(OsStr::to_string_lossy)
                .context("failed to get post file name")?;
            if !self.posts.iter().any(|p| p.slug == slug) {
                let post = Post::new(slug.into_owned(), &self.posts_path, &self.out_path);
                self.posts.push(post);
            }
        }

        Ok(())
    }
}

impl Process for Blog {
    type Input = CompileOptions;

    fn execute(&mut self, opts: &CompileOptions) -> anyhow::Result<()> {
        if opts.clean {
            let _ = fs::remove_dir_all(&self.out_path);
        }

        self.update_posts()?;

        self.copy_public_dir.run()?;

        if opts.dev {
            self.copy_dev_public_dir.run()?;
        }

        let mut parse_results = Vec::with_capacity(self.posts.len());

        for post in &mut self.posts {
            let meta = post
                .parser
                .run()
                .with_context(|| format!("failed to parse post '{}'", post.slug))?;
            parse_results.push(meta.clone())
        }

        let mut processed_results = parse_results
            .into_iter()
            .map(|p| {
                (
                    PostMeta {
                        title: p.meta.title,
                        slug: p.meta.slug,
                        date: p.meta.date,
                        tags: p.meta.tags,
                        prev: p.meta.prev,
                        next: None,
                    },
                    p.events,
                )
            })
            .collect::<Vec<_>>();

        for i in 0..processed_results.len() {
            let Some(prev) = processed_results[i].0.prev.clone() else {
                continue;
            };

            let slug = processed_results[i].0.slug.clone();
            let Some(prev_post) = processed_results.iter_mut().find(|p| p.0.slug == prev) else {
                bail!(
                    "previous post of '{}' not found",
                    processed_results[i].0.slug
                );
            };

            prev_post.0.next = Some(slug);
        }

        let mut metas: Vec<_> = processed_results.iter().map(|p| p.0.clone()).collect();
        for (index, post) in self.posts.iter_mut().enumerate() {
            let parse_result = processed_results[index].clone();
            post.renderer
                .run_with(RenderInput {
                    meta: parse_result.0,
                    events: parse_result.1,
                    metas: metas.clone(),
                    dev: opts.dev,
                })
                .with_context(|| format!("failed to render post '{}'", post.slug))?;

            // if it fails, the post probably doesn't have an images folder!
            let _ = post.copy_images.run();
        }

        metas.sort_by(|a, b| {
            if a.date == b.date {
                a.cmp(b)
            } else {
                // reverse the ordering so that later posts go before earlier posts
                a.date.cmp(&b.date).reverse()
            }
        });

        let index_path = self.out_path.join("index.html");
        let index = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&index_path)
            .with_context(|| format!("failed to open '{}'", index_path.display()))?;

        renderer::write_index(index, &metas, opts.dev)
            .with_context(|| format!("failed to write index page to '{}'", index_path.display()))?;

        let posts_json_path = self.out_path.join("posts.json");
        let posts_json = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&posts_json_path)
            .with_context(|| format!("failed to open '{}'", posts_json_path.display()))?;

        serde_json::to_writer(posts_json, &metas)
            .with_context(|| format!("failed to write to '{}'", posts_json_path.display()))?;

        Ok(())
    }

    paths! {}
}

#[derive(PartialEq, Debug, Clone)]
pub struct CompileOptions {
    pub dev: bool,
    pub clean: bool,
}

#[derive(Serialize, PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
#[serde(into = "SerializedPostMeta")]
pub struct PostMeta {
    pub title: String,
    pub slug: String,
    pub date: Date,
    pub tags: Vec<String>,
    pub prev: Option<String>,
    pub next: Option<String>,
}

#[derive(Serialize)]
pub struct SerializedPostMeta {
    pub title: String,
    pub slug: String,
    pub date: Date,
    pub formatted_date: String,
    pub tags: Vec<String>,
    pub prev: Option<String>,
    pub next: Option<String>,
}

impl From<PostMeta> for SerializedPostMeta {
    fn from(value: PostMeta) -> Self {
        SerializedPostMeta {
            title: value.title,
            slug: value.slug,
            date: value.date,
            formatted_date: value.date.strftime(DATE_FORMAT).to_string(),
            tags: value.tags,
            prev: value.prev,
            next: value.next,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Post {
    slug: String,
    renderer: Processor<Renderer>,
    parser: Processor<Parser>,
    copy_images: Processor<CopyDir>,
}

impl Post {
    pub fn new(slug: String, posts: &Path, dist: &Path) -> Self {
        let in_path = posts.join(&slug);
        let out_path = dist.join(&slug);

        Self {
            slug: slug.clone(),
            copy_images: Processor::new(CopyDir {
                in_path: in_path.join("images"),
                out_path: out_path.join("images"),
            }),
            renderer: Processor::new(Renderer::new(in_path.clone(), out_path.clone())),
            parser: Processor::new(Parser {
                in_path,
                out_path,
                slug,
            }),
        }
    }
}

use anyhow::{Context, bail};
use jiff::civil::Date;
use serde::Serialize;
use std::{
    ffi::OsStr,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
};

use crate::{
    parser::Parser,
    processor::{CopyDir, Process, Processor},
    renderer::{DATE_FORMAT, RenderInput, Renderer},
};
