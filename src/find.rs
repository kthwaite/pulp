use std::{
    ffi::OsStr,
    fs::{self, DirEntry, ReadDir},
    path::{Path, PathBuf},
};

use anyhow::Result;

#[derive(Debug)]
pub struct EbookFinderConfig {
    /// Descend recursively into subdirectories.
    is_recursive: bool,
    /// Ignore hidden files.
    ignore_hidden: bool,
}

impl Default for EbookFinderConfig {
    fn default() -> Self {
        Self {
            is_recursive: true,
            ignore_hidden: true,
        }
    }
}

#[derive(Debug)]
pub struct EbookFinder {
    ///
    config: EbookFinderConfig,
    ///
    dir_stack: Vec<std::io::Result<DirEntry>>,
    ///
    root: PathBuf,
}

impl EbookFinder {
    /// Find ebooks staring from the given path.
    pub fn new_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        if !path.as_ref().is_dir() {
            anyhow::bail!("This is not a directory: {:?}", path.as_ref());
        }
        let path = path.as_ref().to_owned();
        let rd = fs::read_dir(&path)?;
        let mut dir_stack = Vec::default();
        dir_stack.extend(rd);
        Ok(Self {
            root: path,
            dir_stack,
            config: Default::default(),
        })
    }

    /// Check if a path represents a '.epub' file.
    fn is_ebook<P: AsRef<Path>>(&self, path: P) -> bool {
        if path.as_ref().is_dir() {
            return false;
        }
        match path.as_ref().extension().and_then(OsStr::to_str) {
            Some("epub") => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }
}

/// Check if a directory is hidden.
pub fn is_hidden_dir(path: &PathBuf) -> bool {
    path.file_name()
        .and_then(|f| f.to_str())
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

impl Iterator for EbookFinder {
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.dir_stack.pop() {
                Some(Ok(entry)) => {
                    let path = entry.path();
                    if path.is_dir() {
                        if self.config.is_recursive {
                            if self.config.ignore_hidden && is_hidden_dir(&path) {
                                continue;
                            }
                            if let Ok(descend) = fs::read_dir(&path) {
                                self.dir_stack.extend(descend);
                            }
                        }
                        continue;
                    }
                    if self.is_ebook(&path) {
                        return Some(path);
                    }
                }
                Some(Err(_)) => {}
                None => {
                    return None;
                }
            }
        }
    }
}
