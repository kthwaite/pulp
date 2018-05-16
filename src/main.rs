extern crate clap;
extern crate epub;
extern crate select;
extern crate failure;

use clap::{Arg, App};

use epub::doc::EpubDoc;

use select::document::Document;
use select::predicate::Name;

use failure::Error;

use std::path::Path;
use std::io;
use std::io::{Write};


fn get_chapters(book: &mut EpubDoc) -> Result<Vec<Vec<u8>>, Error> {
    book.spine.iter()
        .filter(|res| res.starts_with("chapter"))
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|res| book.get_resource(&res))
        .collect()
}


fn pulp<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    let mut book = EpubDoc::new(path)?;
    let chapters = get_chapters(&mut book);
    let chapters = chapters.expect("oh dear");

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for chapter in &chapters {
        let doc = Document::from(std::str::from_utf8(chapter).unwrap());
        for node in doc.find(Name("p")) {
            // https://github.com/rust-lang/rust/issues/46016
            handle.write_fmt(format_args!("{}\n", node.text())).unwrap();
        }
    };
    Ok(())
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
            Ok(()) => (),
            Err(e) => println!("{}", e)
        }
    };
}
