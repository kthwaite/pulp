use std::{
    ffi::OsStr,
    fs::{self, ReadDir},
    path::{Path, PathBuf},
};

use anyhow::Result;

pub trait EbookFinder: Iterator {
    fn is_ebook<P: AsRef<Path>>(&self, path: P) -> bool;
}

pub struct SimpleEbookFinder {
    path: PathBuf,
    rd: ReadDir,
}

impl SimpleEbookFinder {
    pub fn new_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        if !path.as_ref().is_dir() {
            anyhow::bail!("This is not a directory: {:?}", path.as_ref());
        }
        let path = path.as_ref().to_owned();
        let rd = fs::read_dir(&path)?;
        Ok(Self { path, rd })
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl EbookFinder for SimpleEbookFinder {
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

impl Iterator for SimpleEbookFinder {
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.rd.next().and_then(|r| r.ok()) {
                Some(res) => {
                    let path = res.path();
                    if self.is_ebook(&path) {
                        return Some(path);
                    }
                }
                None => {
                    return None;
                }
            }
        }
    }
}

pub trait EbookDirIter {
    fn iter_ebooks(&self) -> Result<SimpleEbookFinder>;
}

impl<P: AsRef<Path>> EbookDirIter for P {
    fn iter_ebooks(&self) -> Result<SimpleEbookFinder> {
        SimpleEbookFinder::new_from_path(self)
    }
}
