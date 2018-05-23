use extract::get_chapters;
use failure::Error;
use select::document::Document;
use epub::doc::EpubDoc;
use regex::Regex;

use std::io::Write;

use extract::NameRegex;

/// simple chapter output
/// TODO: match on list of tags, rather than just 'p' and 'h\d'
pub fn cat_impl<W: Write>(handle: &mut W, chapters: &[(String, Vec<u8>)], with_headers: bool) -> Result<(), Error> {

    let mut names : Vec<String> = vec![String::from("p")];
    if with_headers {
        names.push(String::from(r"h\d"));
    }
    let names = names.join(r"|");

    let pred = NameRegex::new(names)?;

    for (_res, chapter) in chapters {
        let doc = Document::from(::std::str::from_utf8(chapter).unwrap());
        for node in doc.find(pred.clone()) {
            // https://github.com/rust-lang/rust/issues/46016
            if let Err(error) = handle.write_fmt(format_args!("{}\n", node.text())) {
                if error.kind() == ::std::io::ErrorKind::BrokenPipe {
                    return Ok(())
                }
                else {
                    return Err(error.into());
                }
            }
        }
    };
    Ok(())
}

pub fn grep(mut book: &mut EpubDoc, rx: &Regex) -> Result<bool, Error> {
    let chapters = match get_chapters(&mut book) {
        Ok(p_chapters) => {
            match p_chapters.len() {
                0 => bail!("No chapters found in eBook"),
                _ => p_chapters
            }
        },
        Err(e) => return Err(e)
    };

    let names : Vec<String> = vec![String::from("p")];
    let names = names.join(r"|");

    let pred = NameRegex::new(names)?;

    for (_res, chapter) in chapters {
        let doc = Document::from(::std::str::from_utf8(&chapter).unwrap());
        for node in doc.find(pred.clone()) {
            // https://github.com/rust-lang/rust/issues/46016
            if rx.is_match(&node.text()) {
                return Ok(true);
            }
        }
    };
    Ok(false)
}

pub fn cat(mut book: &mut EpubDoc) -> Result<(), Error> {
    let chapters = match get_chapters(&mut book) {
        Ok(p_chapters) => {
            match p_chapters.len() {
                0 => bail!("No chapters found in eBook"),
                _ => p_chapters
            }
        },
        Err(e) => return Err(e)
    };
    let stdout = ::std::io::stdout();
    let mut handle = stdout.lock();
    cat_impl(&mut handle, &chapters, false)
}

