use std::collections::HashSet;
use std::io::{Read, Seek};
use std::path::PathBuf;

use anyhow::Result;
use epub::doc::{EpubDoc, NavPoint};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Keywords to ignore in resource names.
pub const DEFAULT_IGNORE_RESOURCES: [&str; 25] = [
    r"about(?:the)?author",
    "endpage",
    r"ata\d+",
    "atb",
    "brand",
    r"advert",
    r"also_by",
    r"endpage",
    "cover-image",
    "cover",
    "toc",
    "title",
    "about_book",
    "brief-toc",
    r"bm\d+",
    "dedication",
    "copyright",
    "authorsnote",
    "family_chart",
    "map",
    "picture_section",
    "dedication",
    "acknowledgments",
    "index",
    "notes",
];

/// Keywords to ignore in resource labels.
pub const DEFAULT_IGNORE_LABELS: [&str; 15] = [
    r"Acknowledgements",
    r"Follow Penguin",
    r"Notes",
    r"Works Cited",
    r"About [tT]he (?:Book|Author)",
    r"Image Credits",
    r"Also By",
    r"Acknowledgments",
    r"Extract From",
    r"Have You Read Them All\?",
    r"Copyright",
    r"Contents",
    r"Cover",
    r"Title [P]age",
    r"Dedication",
];

/// Given a Vec of NavPoints containing children, return a flat list of
/// (label, path, order) tuples.
pub fn flatten_navpoints(toc: &[NavPoint]) -> Vec<ResourceInfo> {
    let mut out = vec![];
    for item in toc {
        out.push(item.into());
        out.extend(flatten_navpoints(&item.children));
    }
    out
}

/// Information on an individual resource in an epub.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceInfo {
    pub label: String,
    pub path: PathBuf,
    pub order: usize,
}

impl ResourceInfo {
    pub fn new(label: String, path: PathBuf, order: usize) -> Self {
        Self { label, path, order }
    }
    /// Get the path as a String; tries to call `.to_str()`, falling back on
    /// `.to_string_lossy()` if `None` is returned.
    ///
    // The OCF 2.0.1 §3.3 specifies that paths must be UTF-8, but we are realistic and
    // assume that some paths will be invalid.
    pub fn path_as_string(&self) -> String {
        match self.path.to_str() {
            Some(path) => path.to_string(),
            None => self.path.to_string_lossy().to_string(),
        }
    }
}

impl From<&NavPoint> for ResourceInfo {
    fn from(np: &NavPoint) -> Self {
        Self {
            label: np.label.clone(),
            path: np.content.clone(),
            order: np.play_order.clone(),
        }
    }
}

impl From<NavPoint> for ResourceInfo {
    fn from(np: NavPoint) -> Self {
        Self {
            label: np.label,
            path: np.content,
            order: np.play_order,
        }
    }
}

/// Collection of matched and ignored resources in an epub.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ResourceMatches {
    pub candidates: Vec<ResourceInfo>,
    pub ignored: Vec<ResourceInfo>,
}

impl ResourceMatches {
    /// Return a Vec<ResourceInfo> with each path appearing only once.
    /// NavPoint entries can refer to anchors within a document; thus, we currently
    /// deduplicate documents, taking the first label as the 'canonical' label for
    /// that document.
    pub fn make_unique(mut res: Vec<ResourceInfo>) -> Vec<ResourceInfo> {
        res.sort_by(|a, b| a.order.partial_cmp(&b.order).unwrap());
        let mut disamb: HashSet<String> = Default::default();

        res.into_iter()
            .filter(|item| {
                let path = match item.path.to_str() {
                    Some(path) => path.to_string(),
                    None => item.path.to_string_lossy().to_string(),
                };
                let path = match path.rfind("#") {
                    Some(index) => path[..index].to_string(),
                    None => path,
                };
                if disamb.contains(&path) {
                    false
                } else {
                    disamb.insert(path.to_string());
                    true
                }
            })
            .collect()
    }
    /// Convenience function for calling `make_unique` on `self.candidates`.
    pub fn unique_candidates(&self) -> Vec<ResourceInfo> {
        Self::make_unique(self.candidates.clone())
    }

    /// Return a new `ResourceMatches` where both `candidates` and `ignored` are
    /// unique.
    pub fn into_unique(self) -> Self {
        Self {
            candidates: Self::make_unique(self.candidates),
            ignored: Self::make_unique(self.ignored),
        }
    }
}

///
#[derive(Debug)]
pub struct ResourceExtractorBuilder {
    ignore_labels: Vec<String>,
    ignore_resources: Vec<String>,
}

impl Default for ResourceExtractorBuilder {
    fn default() -> Self {
        Self {
            ignore_labels: DEFAULT_IGNORE_LABELS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            ignore_resources: DEFAULT_IGNORE_RESOURCES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

impl ResourceExtractorBuilder {
    pub fn build(self) -> Result<ResourceExtractor> {
        let rx_str = self.ignore_resources.join(r"|");
        let ignore_resources = Regex::new(&rx_str)?;

        let ignore_labels = self.ignore_labels.join(r"|");
        let ignore_labels = Regex::new(&ignore_labels)?;
        Ok(ResourceExtractor {
            ignore_resources,
            ignore_labels,
        })
    }
}

/// Helper for extracting resources, matching on label or filename.
#[derive(Debug, Clone)]
pub struct ResourceExtractor {
    pub ignore_labels: Regex,
    pub ignore_resources: Regex,
}

impl ResourceExtractor {
    /// Create a new ResourceExtractor from resource and label ignore regex.
    pub fn new(ignore_resources: Regex, ignore_labels: Regex) -> Self {
        Self {
            ignore_resources,
            ignore_labels,
        }
    }

    /// Check if a resource should be ignored.
    pub fn should_ignore(&self, res: &ResourceInfo) -> bool {
        if self.ignore_labels.is_match(&res.label) {
            return true;
        }
        // We hope that the filename will be valid UTF-8, as per the spec, but don't
        // bet on it.
        if let Some(file_name) = res.path.file_name() {
            if let Some(file_str) = file_name.to_str() {
                if self.ignore_resources.is_match(file_str) {
                    return true;
                }
            } else if self.ignore_resources.is_match(&file_name.to_string_lossy()) {
                return true;
            }
        }
        false
    }

    /// Extract matching resources from an epub.
    pub fn extract<R: Read + Seek>(&self, book: &mut EpubDoc<R>) -> Result<ResourceMatches> {
        let mut cands = ResourceMatches::default();

        for item in flatten_navpoints(&book.toc).into_iter() {
            if self.should_ignore(&item) {
                cands.ignored.push(item);
            } else {
                cands.candidates.push(item);
            }
        }
        Ok(cands)
    }
}
