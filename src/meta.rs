use std::collections::HashMap;
use std::io::{Read, Seek};

use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};

const DEFAULT_FIELDS: [&str; 11] = [
    "title",
    "creator",
    "date",
    "identifier",
    "source",
    "publisher",
    "imprint",
    "language",
    "format",
    "fixed-layout",
    "type",
];

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetaVar {
    One(String),
    Many(Vec<String>),
}

/// Create a JSON-serializable table of MetaVars from a book's metadata HashMap.
pub fn meta_vars_from_metadata<R: Read+ Seek>(book: &EpubDoc<R>) -> HashMap<String, MetaVar> {
    book.metadata
        .iter()
        .map(|(key, values)| match values.len() {
            1 => (key.clone(), MetaVar::One(values[0].clone())),
            _ => (key.clone(), MetaVar::Many(values.clone())),
        })
        .collect()
}

#[derive(Debug)]
pub enum OpfMeta {
    Bare(String),
    Scheme { scheme: String, value: String },
}

#[derive(Debug)]
struct DCMeta {
    pub key: String,
    pub value: String,
    pub id: Option<String>,
    pub refines: HashMap<String, OpfMeta>,
}

/// The minimal required metadata that Publications must include consists of three elements from the Dublin Core Metadata Element Set [DCMES] — title, identifier and language — together with the modified property from DCMI Metadata Terms [DCTERMS].
/// In any order: dc:identifier [1 or more], dc:title [1 or more], dc:language [1 or more], DCMES Optional Elements [0 or more], meta [1 or more], OPF2 meta [0 or more], link [0 or more]
/// http://idpf.org/epub/30/spec/epub30-publications.html#sec-meta-elem
#[derive(Debug, Default)]
struct MetadataBuilder {
    pub dc_meta: HashMap<String, DCMeta>,
    pub meta: HashMap<String, OpfMeta>,
    pub refines: HashMap<String, HashMap<String, OpfMeta>>,
    pub ids: HashMap<String, String>,
}

impl MetadataBuilder {
    pub fn from_node<'a>(meta: roxmltree::Node<'a, 'a>) -> Self {
        let mut bld = Self::default();
        for node in meta.children() {
            match node.node_type() {
                roxmltree::NodeType::Element => match node.tag_name().namespace() {
                    Some("http://purl.org/dc/elements/1.1/") => {
                        let meta_el = DCMeta {
                            key: node.tag_name().name().to_string(),
                            value: node.text().unwrap_or("").to_string(),
                            id: node.attribute("id").map(|v| v.to_string()),
                            refines: HashMap::default(),
                        };
                        if let Some(id) = &meta_el.id {
                            bld.ids.insert(id.clone(), meta_el.key.clone());
                        }
                        bld.dc_meta.insert(meta_el.key.clone(), meta_el);
                    }
                    Some("http://www.idpf.org/2007/opf") => {
                        let value = node.text().unwrap_or("").to_string();

                        if let Some(name) = node.attribute("name") {
                            if let Some(content) = node.attribute("content") {
                                bld.meta
                                    .insert(name.to_string(), OpfMeta::Bare(content.to_string()));
                            } else {
                                bld.meta.insert(name.to_string(), OpfMeta::Bare(value));
                            }
                            continue;
                        }
                        let property = node
                            .attribute("property")
                            .expect("<meta> without property")
                            .to_string();

                        match node.attribute("refines") {
                            Some(refine_id) => {
                                let refine_id = refine_id.trim_start_matches('#');
                                if let Some(scheme) = node.attribute("scheme") {
                                    bld.refines
                                        .entry(refine_id.to_string())
                                        .or_insert_with(Default::default)
                                        .insert(
                                            property,
                                            OpfMeta::Scheme {
                                                scheme: scheme.to_string(),
                                                value,
                                            },
                                        );
                                } else {
                                    bld.refines
                                        .entry(refine_id.to_string())
                                        .or_insert_with(Default::default)
                                        .insert(property, OpfMeta::Bare(value));
                                }
                            }
                            None => {
                                if let Some(scheme) = node.attribute("scheme") {
                                    bld.meta.insert(
                                        node.tag_name().name().to_string(),
                                        OpfMeta::Scheme {
                                            scheme: scheme.to_string(),
                                            value,
                                        },
                                    );
                                } else {
                                    bld.meta.insert(
                                        node.tag_name().name().to_string(),
                                        OpfMeta::Bare(value),
                                    );
                                }
                            }
                        }
                    }
                    Some(_) => (),
                    None => (),
                },
                _ => (),
            }
        }
        bld
    }
    pub fn build(mut self) -> Metadata {
        for (key, refines) in self.refines {
            let actual = self.ids.get(&key).unwrap();
            self.dc_meta.get_mut(actual).unwrap().refines = refines;
        }
        let identifier = self.dc_meta.remove("identifier").unwrap();
        let language = self.dc_meta.remove("language").unwrap();
        let creator = self.dc_meta.remove("creator");
        Metadata {
            identifier,
            language,
            creator,
            extra: self.dc_meta,
            meta: self.meta,
            ids: self.ids,
        }
    }
}
#[derive(Debug)]
struct Metadata {
    pub identifier: DCMeta,
    pub language: DCMeta,
    pub creator: Option<DCMeta>,
    pub extra: HashMap<String, DCMeta>,
    pub meta: HashMap<String, OpfMeta>,
    pub ids: HashMap<String, String>,
}

fn from_opf(doc: roxmltree::Document) {
    let ch = doc.root().first_child().unwrap();

    fn find_meta<'a>(ch: roxmltree::Node<'a, 'a>) -> Option<roxmltree::Node<'a, 'a>> {
        for node in ch.children() {
            if node.node_type() == roxmltree::NodeType::Element
                && node.tag_name().name() == "metadata"
            {
                return Some(node);
            }
        }
        None
    }
    let meta = find_meta(ch).unwrap();
}
