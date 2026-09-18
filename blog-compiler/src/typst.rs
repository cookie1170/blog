#[derive(Debug, Clone)]
pub struct TypstCompiler {}

impl TypstCompiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&mut self, math_expr: &str) -> Result<Vec<u8>> {
        let mut typst = Command::new("typst")
            .arg("compile")
            .arg("-")
            .arg("-")
            .args(["--format", "svg"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to spawn typst process")?;

        let mut stdin = typst
            .stdin
            .take()
            .context("typst process should have stdin handle")?;

        stdin.write_fmt(format_args!(
            "#set page(width: auto, height: auto, margin: 0cm); $ {math_expr} $"
        ))?;
        stdin.flush()?;
        typst.stdin = Some(stdin);

        let output = typst
            .wait_with_output()
            .context("failed to wait on typst process")?;

        if !output.status.success() {
            bail!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(output.stdout)
    }
}

use std::{
    io::Write,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
