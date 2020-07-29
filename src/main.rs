mod cat;
mod error;
mod extract;
mod meta;
mod chapter;

use clap::{App, AppSettings, Arg};
use epub::doc::EpubDoc;

use crate::cat::cat;
use crate::error::Error;

fn main() -> Result<(), Error> {
    let app_input = App::new("pulp")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("0.0.1")
        .about("cat epub contents to stdout")
        .arg(Arg::with_name("FILE").required(true))
        .arg(Arg::with_name("meta").long("meta").help("Print ebook metadata and quit").required(false))
        .arg(Arg::with_name("json").long("json").short("json").help("Print output as JSON").required(false))
        .get_matches();

    let path = app_input.value_of("FILE").expect("Must pass FILE");
    let mut book = EpubDoc::new(path).expect("Failed to extract epub");

    if app_input.is_present("meta") {
        for (key, values) in book.metadata {
            match values.len() {
                1 => println!("{}: {}", key, values[0]),
                _ => {
                    println!("{}", key);
                    for val in values {
                        println!("    {}", val);
                    }
                }
            }
        }
        return Ok(())
    }
    cat(&mut book, app_input.is_present("json"))?;
    Ok(())
}
