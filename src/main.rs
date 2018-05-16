extern crate clap;
extern crate epub;
extern crate select;
extern crate failure;
extern crate regex;

use clap::{Arg, App};

use epub::doc::EpubDoc;

use select::document::Document;
use select::predicate::Name;

use failure::Error;

use std::path::Path;
use std::io;
use std::io::{Write};

use regex::Regex;

/// Get chapters from the spine.
/// TODO: Optionally where the ID matches a regex.
/// TODO: With switches for common front- and end-matter.
fn get_chapters(book: &mut EpubDoc) -> Result<Vec<Vec<u8>>, Error> {
    let rx = Regex::new(r"^.*(?:brief-toc|copyright|cover|title).*$|toc").unwrap();
    book.spine.iter()
        .filter(|res| !rx.is_match(res))
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|res| book.get_resource(&res))
        .collect()
}

fn cat(chapters: &Vec<Vec<u8>>) -> Result<(), Error> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for chapter in chapters {
        let doc = Document::from(std::str::from_utf8(chapter).unwrap());
        for node in doc.find(Name("p")) {
            // https://github.com/rust-lang/rust/issues/46016
            if let Err(error) = handle.write_fmt(format_args!("{}\n", node.text())) {
                if error.kind() == std::io::ErrorKind::BrokenPipe {
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

fn pulp<P: AsRef<Path>>(path: P) -> Result<Vec<Vec<u8>>, Error> {
    let mut book = EpubDoc::new(path)?;
    get_chapters(&mut book)
}


fn main() {
    let matches = App::new("pulp")
                    .version("0.0.0")
                    .about("cat for ebook contents")
                    .arg(Arg::with_name("FILE")
                         .required(true))
                    .get_matches();
    if let Some(path) = matches.value_of("FILE") {
        match pulp(path) {
            Ok(chapters) => {
                match chapters.len() {
                    0 => println!("Input file contains no chapters"),
                    _ => cat(&chapters).unwrap()
                }
            },
            Err(e) => println!("{}", e)
        }
    };
}
