#![feature(associated_type_defaults)]

mod parser;
pub mod processor;
mod renderer;
mod typst;

pub const PREFIX: &str = "/blog";

pub struct Blog {
    pub in_path: PathBuf,
    pub out_path: PathBuf,
    copy_public_dir: Processor<CopyDir>,
    copy_dev_public_dir: Processor<CopyDir>,
    posts: Vec<Processor<Post>>,
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
        let posts_path = self.in_path.join("posts");
        self.posts
            .retain(|p| fs::exists(posts_path.join(&p.slug)).is_ok_and(|b| b));

        for post in fs::read_dir(&posts_path)
            .with_context(|| format!("failed to read {}", posts_path.display()))?
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
                let post =
                    Processor::new(Post::new(slug.into_owned(), &posts_path, &self.out_path));
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

        let mut post_metas = Vec::with_capacity(self.posts.len());

        for post in &mut self.posts {
            let post_path = self.posts_path.join(&post.slug);
            let meta = post
                .run_with(opts.dev)
                .with_context(|| format!("failed to compile post at {}", post_path.display()))?;
            post_metas.push(meta)
        }

        let posts_json_path = self.out_path.join("posts.json");
        let posts_json = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&posts_json_path)
            .with_context(|| format!("failed to open {}", posts_json_path.display()))?;

        serde_json::to_writer(posts_json, &post_metas)
            .with_context(|| format!("failed to write to {}", posts_json_path.display()))?;

        Ok(())
    }

    paths! {}
}

#[derive(PartialEq, Debug, Clone)]
pub struct CompileOptions {
    pub dev: bool,
    pub clean: bool,
}

#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct PostMeta {
    pub title: String,
    pub slug: String,
    pub date: Date,
    pub tags: Vec<String>,
}

pub struct Post {
    slug: String,
    in_path: PathBuf,
    out_path: PathBuf,
    renderer: Renderer,
    copy_images: Processor<CopyDir>,
}

impl Post {
    pub fn new(slug: String, posts: &Path, dist: &Path) -> Self {
        let renderer = Renderer::new();
        let in_path = posts.join(&slug);
        let out_path = dist.join(&slug);

        Self {
            slug,
            renderer,
            copy_images: Processor::new(CopyDir {
                in_path: in_path.join("images"),
                out_path: out_path.join("images"),
            }),
            in_path,
            out_path,
        }
    }
}

impl Process for Post {
    type Output = PostMeta;
    type Input = bool;

    fn execute(&mut self, dev: &bool) -> Result<PostMeta> {
        info!("compiling post '{}'", self.slug);

        let markdown = self.in_path.join(&self.slug).with_extension("md");
        let markdown = fs::read_to_string(&markdown)
            .with_context(|| format!("failed to read '{}'", markdown.display()))?;

        let html_path = self.out_path.join("index.html");
        let out_html = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&html_path)
            .with_context(|| format!("failed to open '{}'", html_path.display()))?;

        let out_html = BufWriter::new(out_html);

        let meta = self
            .renderer
            .render(&markdown, out_html, *dev)
            .context("failed to render html")?;

        self.copy_images.run()?;

        Ok(PostMeta {
            title: meta.title,
            slug: self.slug.clone(),
            date: meta.date,
            tags: meta.tags,
        })
    }

    paths! {}
}

use anyhow::{Context, Result};
use jiff::civil::Date;
use serde::Serialize;
use std::{
    ffi::OsStr,
    fs::{self, OpenOptions},
    io::BufWriter,
    path::{Path, PathBuf},
};
use tracing::*;

use crate::{
    processor::{CopyDir, Process, Processor},
    renderer::Renderer,
};
