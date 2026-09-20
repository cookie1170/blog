pub struct Processor<P: Process> {
    pub process: P,
    last_output: Option<P::Output>,
}

pub trait Process {
    type Output = ();

    fn execute(&mut self, in_path: &Path, out_path: &Path) -> Result<Self::Output>;
}

impl<P: Process> Processor<P> {
    pub fn new(process: P) -> Self {
        Self {
            process,
            last_output: None,
        }
    }

    pub fn run(&mut self, in_path: &Path, out_path: &Path) -> Result<&P::Output> {
        let current_hash = hash_directory(in_path)
            .with_context(|| format!("failed to hash '{}'", in_path.display()))?;
        let hash_path = out_path.join("hash.sha256");
        let last_hash = get_last_hash(&hash_path)
            .with_context(|| format!("failed to get last hash of '{}'", out_path.display()))?;
        if last_hash.is_some_and(|h| h == current_hash)
            && let Some(last_output) = self.last_output.as_ref()
        {
            return Ok(last_output);
        }
        fs::create_dir_all(&out_path)
            .with_context(|| format!("failed to create '{}'", out_path.display()))?;

        let out = self.process.execute(in_path, out_path)?;

        fs::write(&hash_path, current_hash)
            .with_context(|| format!("failed to write hash to '{}'", hash_path.display()))?;

        let out = self.last_output.insert(out);
        Ok(out)
    }
}

fn hash_directory(dir: &Path) -> Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    let mut buf = Vec::with_capacity(1024);

    for entry in WalkDir::new(dir) {
        let entry = entry.context("error when walking post directory")?;
        hasher.update(entry.path().as_os_str().as_encoded_bytes());
        if entry.file_type().is_file() {
            buf.clear();
            let mut reader = File::open(entry.path())
                .with_context(|| format!("failed to open '{}'", entry.path().display()))?;
            reader
                .read_to_end(&mut buf)
                .with_context(|| format!("failed to read '{}'", entry.path().display()))?;
            hasher.update(&buf);
        }
    }

    Ok(hasher.finish())
}

fn get_last_hash(hash_path: &Path) -> Result<Option<[u8; 32]>> {
    let result = fs::read(hash_path);
    if let Err(e) = result.as_ref()
        && e.kind() == ErrorKind::NotFound
    {
        return Ok(None);
    }

    let hash = result.with_context(|| format!("failed to read '{}'", hash_path.display()))?;
    hash.try_into()
        .map(Some)
        .ok()
        .context("hash must be 32 bytes")
}

pub struct CopyDir;

impl Process for CopyDir {
    fn execute(&mut self, in_path: &Path, out_path: &Path) -> Result<Self::Output> {
        let _ = fs::remove_dir_all(out_path);
        dircpy::CopyBuilder::new(in_path, out_path)
            .overwrite(true)
            .run()
            .with_context(|| {
                format!(
                    "failed to copy '{}' to '{}'",
                    in_path.display(),
                    out_path.display()
                )
            })
    }
}

use anyhow::{Context as _, Result};
use openssl::sha::Sha256;
use std::{
    fs::{self, File},
    io::{ErrorKind, Read as _},
    path::Path,
};
use walkdir::WalkDir;
