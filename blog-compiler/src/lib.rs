mod parser;
mod renderer;
mod typst;

pub const PREFIX: &str = "/blog";

pub struct Blog {
    pub dist_dir: PathBuf,
    pub posts_dir: PathBuf,
    pub public_dir: PathBuf,
    pub dist_public_dir: PathBuf,
    posts: Vec<Post>,
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
            let post = Post::new(post.path()).context("failed to create post")?;
            self.posts.push(post);
        }

        Ok(())
    }

    pub fn recompile(&mut self) -> anyhow::Result<()> {
        let _ = fs::remove_dir_all(&self.dist_public_dir);
        dircpy::CopyBuilder::new(&self.public_dir, &self.dist_public_dir)
            .overwrite(true)
            .run()
            .with_context(|| {
                format!(
                    "failed to copy '{}' to '{}'",
                    self.public_dir.display(),
                    self.dist_public_dir.display()
                )
            })?;

        for post in &mut self.posts {
            let output_path = self.dist_dir.join(&post.name);
            let _ = fs::remove_dir(&output_path);
            fs::create_dir_all(&output_path).with_context(|| {
                format!(
                    "failed to create output directory '{}'",
                    output_path.display()
                )
            })?;
            let output_path = output_path.clone();
            post.compile(output_path)
                .with_context(|| format!("failed to compile post {}", post.name))?;
        }

        Ok(())
    }
}

pub struct Post {
    name: String,
    path: PathBuf,
    renderer: Renderer,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct PostMeta {
    title: String,
    date: Date,
    tags: Vec<String>,
}

impl Post {
    pub fn new(path: PathBuf) -> Result<Self> {
        let path = path
            .canonicalize()
            .with_context(|| format!("failed to canonicalize '{}'", path.display()))?;
        let name = path
            .file_name()
            .context("failed to get post file name")?
            .to_string_lossy()
            .into_owned();

        let renderer = Renderer::new();

        Ok(Self {
            name,
            path,
            renderer,
        })
    }

    pub fn compile(&mut self, output_path: PathBuf) -> Result<()> {
        let markdown = self.path.join(&self.name).with_extension("md");
        let markdown = fs::read_to_string(&markdown)
            .with_context(|| format!("failed to read '{}'", markdown.display()))?;

        let hash = self.get_current_hash(&markdown);
        let hash_path = output_path.join("hash.sha256");
        let last_hash = self.get_last_hash(&hash_path);
        if last_hash.is_ok_and(|h| hash == h) {
            return Ok(());
        }
        let _ = fs::remove_file(&hash_path);

        info!("compiling post '{}'", self.name);
        let html_path = output_path.join("index.html");
        let out_html = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&html_path)
            .with_context(|| format!("failed to open '{}'", html_path.display()))?;

        let out_html = BufWriter::new(out_html);

        self.renderer
            .render(&markdown, out_html)
            .context("failed to render html")?;

        let images_path = self.path.join("images");
        let dist_images_path = output_path.join("images");
        let _ = fs::remove_dir_all(&dist_images_path);
        dircpy::CopyBuilder::new(&images_path, &dist_images_path)
            .overwrite(true)
            .run()
            .with_context(|| {
                format!(
                    "failed to copy '{}' to '{}'",
                    images_path.display(),
                    dist_images_path.display()
                )
            })?;

        fs::write(&hash_path, hash)
            .with_context(|| format!("failed to write hash to {}", hash_path.display()))?;

        info!("finished compiling post '{}'", self.name);

        Ok(())
    }

    pub fn get_current_hash(&self, markdown: &str) -> [u8; 32] {
        openssl::sha::sha256(markdown.as_bytes())
    }

    pub fn get_last_hash(&self, hash_path: &Path) -> Result<[u8; 32]> {
        fs::read(hash_path)
            .with_context(|| format!("failed to read {}", hash_path.display()))?
            .try_into()
            .ok()
            .context("hash must have 32 bytes")
    }
}

use anyhow::{Context, Result};
use jiff::civil::Date;
use serde::Deserialize;
use std::{
    fs::{self, OpenOptions},
    io::BufWriter,
    path::{Path, PathBuf},
};
use tracing::*;

use crate::renderer::Renderer;
