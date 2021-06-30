use std::collections::HashMap;
use std::io::{Read, Seek};

use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};

// JSON-Serializable representation of book metadata values.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetaVar {
    One(String),
    Many(Vec<String>),
}

/// Create a JSON-serializable table of MetaVars from a book's metadata HashMap.
pub fn meta_vars_from_metadata<R: Read + Seek>(book: &EpubDoc<R>) -> HashMap<String, MetaVar> {
    book.metadata
        .iter()
        .map(|(key, values)| match values.len() {
            1 => (key.clone(), MetaVar::One(values[0].clone())),
            _ => (key.clone(), MetaVar::Many(values.clone())),
        })
        .collect()
}
