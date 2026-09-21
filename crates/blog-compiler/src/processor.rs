#[derive(PartialEq, Debug, Clone)]
pub struct Processor<P: Process> {
    pub process: P,
    last_output: Option<P::Output>,
    last_input: Option<P::Input>,
}

pub trait Process {
    type Output: Debug = ();
    type Input: PartialEq + Debug = ();

    fn execute(&mut self, input: &Self::Input) -> Result<Self::Output>;

    fn in_path(&self) -> &Path;
    fn out_path(&self) -> &Path;
}

impl<P: Process> Processor<P> {
    pub fn new(process: P) -> Self {
        Self {
            process,
            last_output: None,
            last_input: None,
        }
    }

    pub fn run_with(&mut self, input: P::Input) -> Result<&P::Output> {
        let in_path = self.process.in_path();
        let out_path = self.process.out_path();
        let current_hash = hash_directory(in_path, &[&out_path])
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

        let out = self.process.execute(&input)?;

        fs::write(&hash_path, current_hash)
            .with_context(|| format!("failed to write hash to '{}'", hash_path.display()))?;

        self.last_input = Some(input);
        let out = self.last_output.insert(out);
        Ok(out)
    }
}

impl<P: Process<Input = ()>> Processor<P> {
    pub fn run(&mut self) -> Result<&P::Output> {
        self.run_with(())
    }
}

impl<P: Process> DerefMut for Processor<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.process
    }
}

impl<P: Process> Deref for Processor<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.process
    }
}

fn hash_directory(dir: &Path, exclude: &[&Path]) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut buf = Vec::with_capacity(1024);

    'outer: for entry in WalkDir::new(dir).sort_by_file_name() {
        let entry = entry.context("error when walking post directory")?;
        for exclude in exclude {
            if entry.path().starts_with(exclude) {
                continue 'outer;
            }
        }
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

#[derive(PartialEq, Debug, Clone)]
pub struct CopyDir {
    pub in_path: PathBuf,
    pub out_path: PathBuf,
}

#[macro_export]
macro_rules! paths {
    () => {
        fn in_path(&self) -> &Path {
            &self.in_path
        }

        fn out_path(&self) -> &Path {
            &self.out_path
        }
    };
}

impl Process for CopyDir {
    fn execute(&mut self, _: &()) -> Result<Self::Output> {
        let _ = fs::remove_dir_all(&self.out_path);
        dircpy::CopyBuilder::new(&self.in_path, &self.out_path)
            .overwrite(true)
            .run()
            .with_context(|| {
                format!(
                    "failed to copy '{}' to '{}'",
                    self.in_path.display(),
                    self.out_path.display()
                )
            })
    }

    paths! {}
}

use anyhow::{Context as _, Result};
use openssl::sha::Sha256;
use std::{
    fmt::Debug,
    fs::{self, File},
    io::{ErrorKind, Read as _},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
