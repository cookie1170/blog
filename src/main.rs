#![feature(deref_patterns)]

fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new())?;

    let in_path = std::env::current_dir().context("failed to get cwd")?;
    let mut blog = Processor::new(Blog::new(in_path));
    match std::env::args().nth(1) {
        Some(deref!("serve")) => {
            cfg_select! {
                feature = "serve" => {
                    blog_server::serve(&mut blog)?;
                }
                _ => bail!("`serve` cargo feature must be enabled to use `serve`!"),
            }
        }
        Some(deref!("compile")) => {
            blog.run_with(CompileOptions {
                dev: false,
                clean: true,
            })?;
        }
        Some(other) => bail!("unknown action: '{other}'"),
        None => bail!("expected action argument"),
    }

    Ok(())
}

use anyhow::{Context, bail};
use blog_compiler::{Blog, CompileOptions, processor::Processor};
