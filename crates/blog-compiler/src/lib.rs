#![feature(associated_type_defaults)]

mod parser;
mod processor;
mod renderer;
mod typst;

pub const PREFIX: &str = "/blog";

pub struct Blog {
    pub dist_dir: PathBuf,
    pub posts_dir: PathBuf,
    pub public_dir: PathBuf,
    pub dist_public_dir: PathBuf,
    copy_public_dir: Processor<CopyDir>,
    posts: Vec<Processor<Post>>,
}

impl Blog {
    pub fn new(root: PathBuf) -> anyhow::Result<Self> {
        let root = root
            .canonicalize()
            .with_context(|| format!("failed to canonicalize '{}'", root.display()))?;
        let posts_dir = root.join("posts");
        let dist_dir = root.join("dist");
        let public_dir = root.join("public");
        let dist_public_dir = dist_dir.join("public");
        let mut blog = Self {
            posts_dir,
            dist_dir,
            public_dir,
            dist_public_dir,
            copy_public_dir: Processor::new(CopyDir),
            posts: Vec::new(),
        };
        blog.update_posts()?;
        Ok(blog)
    }

    pub fn update_posts(&mut self) -> anyhow::Result<()> {
        self.posts.clear();
        for post in fs::read_dir(&self.posts_dir)
            .with_context(|| format!("failed to read {}", self.posts_dir.display()))?
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
            let name = path
                .file_name()
                .map(OsStr::to_string_lossy)
                .context("failed to get post file name")?;
            let post = Processor::new(Post::new(name.into_owned()));
            self.posts.push(post);
        }

        Ok(())
    }

    pub fn recompile(&mut self, clean: bool) -> anyhow::Result<()> {
        if clean {
            let _ = fs::remove_dir_all(&self.dist_dir);
        }

        self.copy_public_dir
            .run(&self.public_dir, &self.dist_public_dir)?;

        for post in &mut self.posts {
            let output_path = self.dist_dir.join(&post.process.name);
            let input_path = self.posts_dir.join(&post.process.name);
            post.run(&input_path, &output_path)
                .with_context(|| format!("failed to compile post at {}", input_path.display()))?;
        }

        Ok(())
    }
}

pub struct Post {
    name: String,
    renderer: Renderer,
    copy_images: Processor<CopyDir>,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct PostMeta {
    title: String,
    date: Date,
    tags: Vec<String>,
}

impl Post {
    pub fn new(name: String) -> Self {
        let renderer = Renderer::new();

        Self {
            name,
            renderer,
            copy_images: Processor::new(CopyDir),
        }
    }
}

impl Process for Post {
    type Output = PostMeta;

    fn execute(&mut self, in_path: &Path, out_path: &Path) -> Result<PostMeta> {
        info!("compiling post '{}'", self.name);

        let markdown = in_path.join(&self.name).with_extension("md");
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
            .render(&markdown, out_html)
            .context("failed to render html")?;

        let images_path = in_path.join("images");
        let dist_images_path = out_path.join("images");
        self.copy_images.run(&images_path, &dist_images_path)?;

        Ok(meta)
    }
}

use anyhow::{Context, Result};
use jiff::civil::Date;
use serde::Deserialize;
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
