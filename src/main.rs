mod cat;
mod chapter;
mod error;
mod extract;
mod meta;

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
        .arg(
            Arg::with_name("meta")
                .long("meta")
                .help("Print ebook metadata and quit")
                .required(false),
        )
        .arg(
            Arg::with_name("json")
                .long("json")
                .short("json")
                .help("Print output as JSON")
                .required(false),
        )
        .get_matches();

    let path = app_input.value_of("FILE").expect("Must pass FILE");
    let mut book = EpubDoc::new(path).expect("Failed to extract epub");

    if app_input.is_present("meta") {
        let stdout = ::std::io::stdout();
        let mut handle = stdout.lock();
        let map = meta::meta_vars_from_metadata(&book);
        serde_json::to_writer(handle, &map).map_err(Error::JsonError)
    } else {
        cat(&mut book, app_input.is_present("json"))
    }
}
