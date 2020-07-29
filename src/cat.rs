use epub::doc::EpubDoc;
use std::io::Write;

use crate::error::Error;
use crate::extract::get_chapters;

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

pub fn cat_plain<W: Write>(
    handle: &mut W,
    chapters: &[(String, Vec<u8>)],
) -> Result<(), Error> {
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

pub fn cat(mut book: &mut EpubDoc) -> Result<(), Error> {
    let chapters = get_chapters(&mut book).and_then(|p_chapters| match p_chapters.len() {
        0 => Err(Error::NoChapters),
        _ => Ok(p_chapters),
    })?;
    let stdout = ::std::io::stdout();
    let mut handle = stdout.lock();
    cat_plain(&mut handle, &chapters)
}
