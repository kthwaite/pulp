extern crate clap;
extern crate epub;
#[macro_use]
extern crate failure;
extern crate glob;
extern crate regex;
extern crate select;

mod extract;
mod cat;
mod meta;

use clap::{Arg, App, SubCommand};
use epub::doc::EpubDoc;
use regex::Regex;
use std::path::PathBuf;

use cat::cat;
use meta::list_meta;


fn main() {
    let meta_cmd = SubCommand::with_name("meta")
                        .about("print ebook meta to stdout")
                        .arg(Arg::with_name("FILE")
                             .required(true))
                        .arg(Arg::with_name("fields")
                             .long("fields")
                             .short("f")
                             .takes_value(true))
                        .arg(Arg::with_name("fieldnames")
                             .long("with-fieldnames")
                             .short("w")
                             .takes_value(false));

    let cat_cmd = SubCommand::with_name("cat")
                        .about("cat epub contents to stdout")
                        .arg(Arg::with_name("FILE")
                             .required(true))
                        .arg(Arg::with_name("match-paths")
                             .short("p")
                             .takes_value(true))
                        .arg(Arg::with_name("match-ids")
                             .short("i")
                             .takes_value(true));

    let grep_cmd = SubCommand::with_name("grep")
                        .arg(Arg::with_name("EXPR")
                             .required(true))
                        .arg(Arg::with_name("path")
                             .default_value("."));

    let matches = App::new("pulp")
                    .version("0.0.1")
                    .about("ebook multitool")
                    .subcommand(cat_cmd)
                    .subcommand(meta_cmd)
                    .subcommand(grep_cmd)
                    .get_matches();


    if let Some(matches) = matches.subcommand_matches("cat") {
        let path = matches.value_of("FILE").unwrap();
        let mut book = EpubDoc::new(path).unwrap();
        match cat(&mut book) {
            Ok(_) => (),
            Err(e) => println!("{}", e)
        }
    };

    if let Some(matches) = matches.subcommand_matches("grep") {
        let regex = matches.value_of("EXPR").unwrap();
        let regex = Regex::new(regex).unwrap();
        let glob_expr = matches.value_of("path").unwrap();

        let ebooks = glob::glob(glob_expr).unwrap()
                                .filter(|path| path.is_ok())
                                .map(|path| path.unwrap())
                                .filter(|path| path.extension().is_some())
                                .filter(|path| path.extension().unwrap() == "epub")
                                .collect::<Vec<PathBuf>>();
        if ebooks.len() == 0 {
            println!("No files found or invalid glob expression");
            return;
        }
        for path in ebooks {
            match EpubDoc::new(&path) {
                Ok(mut book) => {
                    if cat::grep(&mut book, &regex).unwrap() {
                        println!("{}", path.as_path().to_str().unwrap());
                    }
                },
                Err(_error) => () // println!("{:?}", e)
            }
        }
    }

    if let Some(matches) = matches.subcommand_matches("meta") {
        let path = matches.value_of("FILE").unwrap();
        let with_fieldnames = matches.is_present("fieldnames");
        let mut book = EpubDoc::new(path).unwrap();
        match matches.value_of("fields") {
            Some(fields) => {
                let fields : Vec<String> = fields.split(",")
                                                .into_iter()
                                                .map(|s| s.to_owned())
                                                .collect();
                list_meta(&mut book, Some(&fields), with_fieldnames).unwrap()
            },
            None => list_meta(&mut book, None, with_fieldnames).unwrap()
        };
    }
}
