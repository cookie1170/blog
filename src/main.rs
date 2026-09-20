#![feature(deref_patterns)]

fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new())?;

    let mut blog = Blog::new(std::env::current_dir().context("failed to get cwd")?)?;
    match std::env::args().nth(1) {
        Some(deref!("serve")) => {
            cfg_select! {
                feature = "serve" => blog_server::serve(&mut blog),
                _ => bail!("`serve` cargo feature must be enabled to use `serve`!"),
            }
        }
        Some(deref!("compile")) => blog.recompile(CompileOptions {
            dev: false,
            clean: true,
        }),
        Some(other) => bail!("unknown action: '{other}'"),
        None => bail!("expected action argument"),
    }
}

use anyhow::{Context, bail};
use blog_compiler::{Blog, CompileOptions};
