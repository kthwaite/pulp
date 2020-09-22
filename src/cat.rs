use epub::doc::EpubDoc;
use std::io::{Read, Seek, Write};

use crate::book::{Book, Chapter};
use crate::error::Error;
use crate::extract::get_chapters;
use crate::meta::{meta_vars_from_metadata, MetaVar};

fn write_check_pipe<W: Write>(handle: &mut W, text: &str) -> Result<(), Error> {
    if let Err(error) = handle.write_fmt(format_args!("{}", text)) {
        if error.kind() == ::std::io::ErrorKind::BrokenPipe {
            return Ok(());
        } else {
            return Err(error.into());
        }
    }
    Ok(())
}

pub fn cat_plain_recursive<'a, W: Write>(
    handle: &mut W,
    node: roxmltree::Node<'a, 'a>,
) -> Result<(), Error> {
    for desc in node.descendants().skip(1) {
        match desc.node_type() {
            roxmltree::NodeType::Text => write_check_pipe(handle, desc.text().unwrap_or(""))?,
            roxmltree::NodeType::Element => match desc.tag_name().name() {
                // TODO: anything else?
                "br" => write_check_pipe(handle, "\n")?,
                _ => (),
            },
            _ => (),
        }
    }
    match node.tag_name().name() {
        "h1" | "h2" | "p" => write_check_pipe(handle, "\n"),
        _ => Ok(()),
    }
}

pub fn cat_plain<W: Write>(handle: &mut W, chapters: &[(String, Vec<u8>)]) -> Result<(), Error> {
    'chapter_iter: for (_res, chapter) in chapters {
        let doc = roxmltree::Document::parse(::std::str::from_utf8(chapter).unwrap()).unwrap();
        let root = match doc.root().first_child() {
            None => continue 'chapter_iter,
            Some(root_node) => root_node,
        };
        for node in root.children() {
            if let "body" = node.tag_name().name() {
                for child in node.children() {
                    cat_plain_recursive(handle, child)?
                }
            }
        }
        write_check_pipe(handle, "\n")?;
    }
    Ok(())
}

pub fn cat_span_recursive<'a, W: Write>(
    handle: &mut W,
    node: roxmltree::Node<'a, 'a>,
) -> Result<(), Error> {
    for desc in node.descendants().skip(1) {
        match desc.node_type() {
            roxmltree::NodeType::Text => write_check_pipe(handle, desc.text().unwrap_or(""))?,
            roxmltree::NodeType::Element => match desc.tag_name().name() {
                // TODO: anything else?
                "br" => write_check_pipe(handle, "\n")?,
                _ => (),
            },
            _ => (),
        }
    }
    match node.tag_name().name() {
        "h1" | "h2" | "p" => write_check_pipe(handle, "\n"),
        _ => Ok(()),
    }
}

pub fn cat_json(chapters: &[(String, Vec<u8>)]) -> Result<Vec<Chapter>, Error> {
    let mut chaps = Vec::<Chapter>::new();
    'chapter_iter: for (res, chapter) in chapters {
        let doc = roxmltree::Document::parse(::std::str::from_utf8(chapter).unwrap()).unwrap();
        let root = match doc.root().first_child() {
            None => continue 'chapter_iter,
            Some(root_node) => root_node,
        };
        let mut chap = Chapter::default().with_res(res);
        let mut first_header = true;
        for node in root.children() {
            if let "body" = node.tag_name().name() {
                for child in node.children() {
                    match child.tag_name().name() {
                        "h1" | "h2" => {
                            if first_header {
                                for desc in child.descendants() {
                                    if desc.node_type() == roxmltree::NodeType::Text {
                                        chap.header.push_str(desc.text().unwrap_or(""));
                                    }
                                }
                                first_header = false;
                            } else {
                                let mut head = String::new();
                                for desc in child.descendants() {
                                    if desc.node_type() == roxmltree::NodeType::Text {
                                        head.push_str(desc.text().unwrap_or(""));
                                    }
                                }
                                chap.spans.push(head);
                            }
                        }
                        "p" => {
                            let mut text = String::new();
                            for desc in child.descendants() {
                                if desc.node_type() == roxmltree::NodeType::Text {
                                    text.push_str(desc.text().unwrap_or(""));
                                }
                            }
                            chap.spans.push(text);
                        }
                        _ => (),
                    }
                }
            }
        }
        chaps.push(chap);
    }
    Ok(chaps)
}

pub fn cat<R: Read + Seek>(mut book: &mut EpubDoc<R>, as_json: bool) -> Result<(), Error> {
    let raw_chapters = get_chapters(&mut book).and_then(|p_chapters| match p_chapters.len() {
        0 => Err(Error::NoChapters),
        _ => Ok(p_chapters),
    })?;
    let stdout = ::std::io::stdout();
    let mut handle = stdout.lock();
    if as_json {
        let chapters = cat_json(&raw_chapters)?;
        let book = Book {
            meta: meta_vars_from_metadata(&book),
            chapters,
        };
        serde_json::to_writer(handle, &book).map_err(Error::JsonError)
    } else {
        cat_plain(&mut handle, &raw_chapters)
    }
}
