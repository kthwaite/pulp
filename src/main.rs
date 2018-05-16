extern crate clap;
extern crate epub;
extern crate select;
extern crate failure;
extern crate regex;

use clap::{Arg, App};

use epub::doc::EpubDoc;

use select::document::Document;
use select::predicate::Predicate;
use select::node::Node;

use failure::Error;

use std::path::Path;
use std::io;
use std::io::{Write};

use regex::Regex;

/// Matches Element Node name by regex.
#[derive(Clone, Debug)]
struct NameRegex {
    rx: regex::Regex
}

impl NameRegex {
    fn new<T: AsRef<str>>(rx_str: T) -> Result<Self, Error> {
        let rx = regex::Regex::new(rx_str.as_ref())?;
        Ok(NameRegex { rx })
    }
}

impl Predicate for NameRegex {
    fn matches(&self, node: &Node) -> bool {
        match node.name() {
            Some(name) => self.rx.is_match(name),
            None => false
        }
    }
}


/// Get chapters from the spine.
/// TODO: Optionally where the ID matches a regex.
/// TODO: With switches for common front- and end-matter.
fn get_chapters(book: &mut EpubDoc) -> Result<Vec<Vec<u8>>, Error> {
    let ignore = Regex::new(r"^.*(?:brief-toc|copyright|cover|title).*$|toc").unwrap();
    book.spine.iter()
        .filter(|res| !ignore.is_match(res))
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|res| book.get_resource(&res))
        .collect()
}


/// simple chapter output
/// TODO: match on list of tags, rather than just 'p' and 'h\d'
fn cat(chapters: &Vec<Vec<u8>>, with_headers: bool) -> Result<(), Error> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let mut names : Vec<String> = vec![String::from("p")];
    if with_headers {
        names.push(String::from(r"h\d"));
    }
    let names = names.join(r"|");

    let pred = NameRegex::new(names)?;

    for chapter in chapters {
        let doc = Document::from(std::str::from_utf8(chapter).unwrap());
        for node in doc.find(pred.clone()) {
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
                    _ => cat(&chapters, true).unwrap()
                }
            },
            Err(e) => println!("{}", e)
        }
    };
}
