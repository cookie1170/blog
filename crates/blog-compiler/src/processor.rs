pub struct Processor<P: Process> {
    pub process: P,
    last_output: Option<P::Output>,
    last_input: Option<P::Input>,
}

pub trait Process {
    type Output = ();
    type Input: PartialEq = ();

    fn execute(
        &mut self,
        input: &Self::Input,
        in_path: &Path,
        out_path: &Path,
    ) -> Result<Self::Output>;
}

impl<P: Process> Processor<P> {
    pub fn new(process: P) -> Self {
        Self {
            process,
            last_output: None,
            last_input: None,
        }
    }

    pub fn run_with(
        &mut self,
        input: P::Input,
        in_path: &Path,
        out_path: &Path,
    ) -> Result<&P::Output> {
        let current_hash = hash_directory(in_path)
            .with_context(|| format!("failed to hash '{}'", in_path.display()))?;
        let hash_path = out_path.join("hash.txt");
        let last_hash = get_last_hash(&hash_path)
            .with_context(|| format!("failed to get last hash of '{}'", out_path.display()))?;
        if last_hash.is_some_and(|h| h == current_hash)
            && let Some(last_output) = self.last_output.as_ref()
            && let Some(last_input) = self.last_input.as_ref()
            && *last_input == input
        {
            return Ok(last_output);
        }
        fs::create_dir_all(&out_path)
            .with_context(|| format!("failed to create '{}'", out_path.display()))?;

        let out = self.process.execute(&input, in_path, out_path)?;

        fs::write(&hash_path, current_hash)
            .with_context(|| format!("failed to write hash to '{}'", hash_path.display()))?;

        self.last_input = Some(input);
        let out = self.last_output.insert(out);
        Ok(out)
    }
}

impl<P: Process<Input = ()>> Processor<P> {
    pub fn run(&mut self, in_path: &Path, out_path: &Path) -> Result<&P::Output> {
        self.run_with((), in_path, out_path)
    }
}

fn hash_directory(dir: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut buf = Vec::with_capacity(1024);

    for entry in WalkDir::new(dir).sort_by_file_name() {
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

    Ok(hash_to_string(hasher.finish()))
}

fn get_last_hash(hash_path: &Path) -> Result<Option<String>> {
    let result = fs::read_to_string(hash_path);
    if let Err(e) = result.as_ref()
        && e.kind() == ErrorKind::NotFound
    {
        return Ok(None);
    }

    result
        .map(Some)
        .with_context(|| format!("failed to read '{}'", hash_path.display()))
}

fn hash_to_string(hash: [u8; 32]) -> String {
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub struct CopyDir;

impl Process for CopyDir {
    fn execute(&mut self, _: &(), in_path: &Path, out_path: &Path) -> Result<Self::Output> {
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
