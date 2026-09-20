#[derive(Debug, Clone)]
pub struct TypstCompiler {}

impl TypstCompiler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn compile(&mut self, math_expr: &str) -> Result<String> {
        let mut typst = Command::new("typst")
            .arg("compile")
            .arg("-")
            .arg("-")
            .args(["--format", "svg"])
            .args(["--font-path", "public/fonts"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to spawn typst process")?;

        let mut stdin = typst
            .stdin
            .take()
            .context("typst process should have stdin handle")?;

        stdin.write(format!(
            r#"#set page(width: auto, height: auto, margin: 0cm); #show math.equation: set text(font: "Fira Math"); $ {math_expr} $"#
        ).as_bytes()).await?;
        stdin.flush().await?;
        typst.stdin = Some(stdin);

        let output = typst
            .wait_with_output()
            .await
            .context("failed to wait on typst process")?;

        if !output.status.success() {
            bail!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(String::from_utf8_lossy_owned(output.stdout))
    }
}

use std::process::Stdio;
use tokio::{io::AsyncWriteExt, process::Command};

use anyhow::{Context, Result, bail};
