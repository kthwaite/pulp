use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek};
use std::path::PathBuf;

use anyhow::{Context, Result};
use epub::doc::EpubDoc;
use quick_xml::{events::Event, Reader};
use serde::{Deserialize, Serialize};

use crate::{
    extract::{ResourceExtractorBuilder, ResourceInfo},
    meta::MetaVar,
};

/// Simple representation of book chapter content.
#[derive(Debug, Deserialize, Serialize)]
pub struct SimpleChapter {
    ///
    doc: String,
    /// Label in toc.ncx, if any
    label: String,
    /// First header in the document, if any.
    header: Option<String>,
    /// List of strings representing text content.
    content: Vec<String>,
}

impl std::convert::From<SimpleAggregator> for SimpleChapter {
    fn from(tra: SimpleAggregator) -> Self {
        SimpleChapter {
            doc: tra.doc,
            label: tra.label,
            header: tra.first_header,
            content: tra.spans,
        }
    }
}

/// Simple representation of book content broken down by chapter.
#[derive(Debug, Deserialize, Serialize)]
pub struct SimpleBook {
    meta: HashMap<String, MetaVar>,
    chapters: Vec<SimpleChapter>,
}

/// Simple aggregator for content in an XHTML document.
#[derive(Debug, Deserialize, Serialize)]
pub struct SimpleAggregator {
    doc: String,
    label: String,
    first_header: Option<String>,
    spans: Vec<String>,
    current: String,
}

impl SimpleAggregator {
    /// Create a new `SimpleAggregator`.
    pub fn new(doc: String, label: String) -> Self {
        Self {
            doc,
            label,
            first_header: None,
            spans: vec![],
            current: String::new(),
        }
    }
    /// Push text into the current span.
    pub fn push_str(&mut self, text: &str) {
        self.current.push_str(text);
    }

    /// Push the current span into the list of spans, and allocate a new 'current'
    /// span.
    pub fn push_span(&mut self) {
        // Don't bother reallocating if the current string is composed only of whitespace.
        if self.current.trim().len() == 0 {
            return;
        }
        let mut tmp = String::new();
        std::mem::swap(&mut self.current, &mut tmp);
        self.spans.push(tmp.trim().to_string());
    }

    /// Push the current span into the list of spans, unless no 'first_header' exists,
    /// in which case set this as the first header.
    pub fn push_header(&mut self) {
        let mut tmp = String::new();
        std::mem::swap(&mut self.current, &mut tmp);
        let tmp = tmp.trim().to_string();
        if self.first_header.is_none() {
            self.first_header = Some(tmp);
        } else {
            self.spans.push(tmp);
        }
    }
}

/// Read an individual XHTML document, extracting text to produce a SimpleChapter.
pub fn read_content_simple(
    doc: String,
    label: String,
    data: Vec<u8>,
    buf: &mut Vec<u8>,
    custom_entities: &HashMap<Vec<u8>, Vec<u8>>,
) -> Result<SimpleChapter> {
    let mut rdr = Reader::from_reader(Cursor::new(data));
    let mut tra = SimpleAggregator::new(doc, label);
    // Skip over the <head>
    'read_head: loop {
        match rdr.read_event(buf)? {
            Event::End(ref e) => {
                if e.name() == b"head" {
                    break 'read_head;
                }
            }
            Event::Eof => break 'read_head,
            _ => (),
        }
    }
    // Read text from elements in the <body>
    'read_body: loop {
        match rdr.read_event(buf)? {
            Event::Text(ref e) => {
                let unescaped = e.unescaped_with_custom_entities(custom_entities)?;
                let text = rdr.decode(&unescaped)?;
                tra.push_str(text);
            }
            Event::Start(ref e) => {
                match e.name() {
                    // silently consume <a>, <sub>, <sup>
                    // TODO: ... but not inside headers.
                    // TODO: ... and be smarter about this (parsing attributes, etc)
                    b"a" => 'consume_anchor: loop {
                        match rdr.read_event(buf)? {
                            Event::Eof => todo!(),
                            Event::End(ref e) => {
                                if e.name() == b"a" {
                                    break 'consume_anchor;
                                }
                            }
                            _ => (),
                        }
                    },
                    _ => {}
                }
            }
            Event::End(ref e) => match e.name() {
                b"div" | b"section" | b"p" => {
                    tra.push_span();
                }
                b"h1" | b"h2" | b"h3" | b"h4" | b"h5" | b"h6" => {
                    tra.push_header();
                }
                _ => (),
            },
            Event::Empty(ref e) => {
                if e.name() == b"br" {
                    tra.push_str("\n");
                }
            }
            Event::Comment(_) => (),
            Event::CData(_) => (),
            Event::Decl(_) => (),
            Event::PI(_) => (),
            Event::DocType(_) => (),
            Event::Eof => break 'read_body,
        }
    }
    Ok(tra.into())
}

// FIXME: refactor and remove
fn unhash_path(item: &ResourceInfo) -> Cow<str> {
    let path = match item.path.to_str() {
        Some(path) => path.to_string(),
        None => item.path.to_string_lossy().to_string(),
    };
    let path = match path.rfind("#") {
        Some(index) => Cow::from(path[..index].to_string()),
        None => Cow::from(path),
    };
    path
}

/// Transform a string literal mapping into a HashMap mapping bytes to bytes.
macro_rules! bytes_map {
    { $($key: expr => $value: expr),+ $(,)* } => {
                                               {
        let mut _hashmap : HashMap<Vec<u8>, Vec<u8>> = Default::default();
        $( _hashmap.insert($key.as_bytes().to_vec(), $value.as_bytes().to_vec()); )+
        _hashmap
                                               }
    }
}

/// Get the default custom entities for unencoding ebook text.
pub fn default_custom_entities() -> HashMap<Vec<u8>, Vec<u8>> {
    bytes_map! {
        "nbsp" => "",
        "mdash" => "—",
        "ndash" => "–",
        "iquest" => "¿",
        "lsquo" => "‘",
        "rsquo" => "’",
        "middot" => "·",
        "shy" => "-",
        "eacute" => "é",
        "acirc" => "â",
        "auml" => "ä",
        "ldquo" => "“",
        "rdquo" => "”",
        "Eacute" => "É",
    }
}

/// Iterate over chapters in a book, creating a SimpleBook.
pub fn transform_simple<R: Read + Seek>(book: &mut EpubDoc<R>) -> Result<SimpleBook> {
    let meta_map = crate::meta::meta_vars_from_metadata(book);

    let ext = ResourceExtractorBuilder::default().build()?;
    let extracted = ext.extract(book)?;
    let mut buf = Vec::default();

    let custom_entities = default_custom_entities();
    let parsed_chapters = extracted
        .unique_candidates()
        .into_iter()
        .map(|item| -> Result<SimpleChapter> {
            let path = unhash_path(&item);
            let data = book
                .get_resource_by_path(&*path)
                .with_context(|| format!("Failed to get resource: {:?}", item))?;
            read_content_simple(
                item.path_as_string().clone(),
                item.label.clone(),
                data,
                &mut buf,
                &custom_entities,
            ).with_context(|| format!("Failed to parse resource: {:?}", item))
        })
        .collect::<Result<Vec<SimpleChapter>>>()?;
    Ok(SimpleBook {
        meta: meta_map,
        chapters: parsed_chapters,
    })
}
