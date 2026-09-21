#![feature(associated_type_defaults)]

mod parser;
pub mod processor;
mod renderer;
mod typst;

pub const PREFIX: &str = "/blog";

pub struct Blog {
    copy_public_dir: Processor<CopyDir>,
    copy_dev_public_dir: Processor<CopyDir>,
    posts: Vec<Processor<Post>>,
}

impl Blog {
    pub fn new() -> Self {
        Self {
            copy_public_dir: Processor::new(CopyDir),
            copy_dev_public_dir: Processor::new(CopyDir),
            posts: Vec::new(),
        }
    }

    pub fn update_posts(&mut self, posts_dir: &Path) -> anyhow::Result<()> {
        self.posts
            .retain(|p| fs::exists(posts_dir.join(&p.slug)).is_ok_and(|b| b));

        for post in fs::read_dir(posts_dir)
            .with_context(|| format!("failed to read {}", posts_dir.display()))?
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
                let post = Processor::new(Post::new(slug.into_owned()));
                self.posts.push(post);
            }
        }

        Ok(())
    }
}

impl Process for Blog {
    type Input = CompileOptions;

    fn execute(
        &mut self,
        opts: &CompileOptions,
        in_path: &Path,
        out_path: &Path,
    ) -> anyhow::Result<()> {
        if opts.clean {
            let _ = fs::remove_dir_all(out_path);
        }

        let posts_path = in_path.join("posts");
        self.update_posts(&posts_path)?;

        self.copy_public_dir
            .run(in_path.join("public"), out_path.join("public"))?;

        if opts.dev {
            self.copy_dev_public_dir
                .run(in_path.join("dev_public"), out_path.join("dev_public"))?;
        }

        let mut post_metas = Vec::with_capacity(self.posts.len());

        for post in &mut self.posts {
            let in_path = posts_path.join(&post.slug);
            let out_path = out_path.join(&post.slug);
            let meta = post
                .run_with(opts.dev, &in_path, &out_path)
                .with_context(|| format!("failed to compile post at {}", in_path.display()))?;
            post_metas.push(meta)
        }

        let posts_json_path = out_path.join("posts.json");
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
    renderer: Renderer,
    copy_images: Processor<CopyDir>,
}

impl Post {
    pub fn new(slug: String) -> Self {
        let renderer = Renderer::new();

        Self {
            slug,
            renderer,
            copy_images: Processor::new(CopyDir),
        }
    }
}

impl Process for Post {
    type Output = PostMeta;
    type Input = bool;

    fn execute(&mut self, dev: &bool, in_path: &Path, out_path: &Path) -> Result<PostMeta> {
        info!("compiling post '{}'", self.slug);

        let markdown = in_path.join(&self.slug).with_extension("md");
        let markdown = fs::read_to_string(&markdown)
            .with_context(|| format!("failed to read '{}'", markdown.display()))?;

        let html_path = out_path.join("index.html");
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

        let images_path = in_path.join("images");
        let dist_images_path = out_path.join("images");
        self.copy_images.run(&images_path, &dist_images_path)?;

        Ok(PostMeta {
            title: meta.title,
            slug: self.slug.clone(),
            date: meta.date,
            tags: meta.tags,
        })
    }
}

use anyhow::{Context, Result};
use jiff::civil::Date;
use serde::Serialize;
use std::{
    ffi::OsStr,
    fs::{self, OpenOptions},
    io::BufWriter,
    path::Path,
};
use tracing::*;

use crate::{
    processor::{CopyDir, Process, Processor},
    renderer::Renderer,
};
