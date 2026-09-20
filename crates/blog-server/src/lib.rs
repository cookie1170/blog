#![feature(trim_prefix_suffix)]

#[tokio::main]
pub async fn serve(blog: &mut Blog) -> anyhow::Result<()> {
    if let Err(e) = blog.recompile(CompileOptions {
        dev: true,
        clean: false,
    }) {
        error!("{e:?}");
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = tx.send(event);
    })
    .context("failed to intialise filesystem watcher")?;

    watcher
        .watch(&blog.root_dir, RecursiveMode::Recursive)
        .with_context(|| format!("failed to watch '{}'", blog.root_dir.display()))?;
    info!("watching {}", blog.root_dir.display());

    let server = warp::serve(warp::path(PREFIX.trim_prefix('/')).and(warp::fs::dir("dist")));
    tokio::select! {
        _ = async {
                while let Some(event) = rx.recv().await {
                    let event = match event {
                        Ok(event) => event,
                        Err(e) => {
                            error!("error when receiving notify event: {e}");
                            continue;
                        }
                    };
                    if !matches!(event.kind, EventKind::Modify(..) | EventKind::Create(..) | EventKind::Remove(..)) {
                        continue;
                    }
                    if event.paths.iter().any(|p| p.starts_with(&blog.dist_dir)) {
                        continue;
                    }
                    if let Err(e) = blog.recompile(CompileOptions { dev: true, clean: false }) {
                        error!("{e:?}");
                    }
                }
        } => (),
        _ = async {
            let addr = "127.0.0.1:8000";
            info!("serving at {addr}");
            server.run(addr.parse::<SocketAddrV4>().unwrap()).await;
        } => (),
    };

    Ok(())
}

use std::net::SocketAddrV4;

use anyhow::Context as _;
use blog_compiler::{Blog, CompileOptions, PREFIX};
use notify::{EventKind, RecursiveMode, Watcher};
use tracing::{error, info};
use warp::Filter as _;
