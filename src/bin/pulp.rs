use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use epub::doc::EpubDoc;

use pulp::cat::simple::transform_simple;
use pulp::extract::{flatten_navpoints, ResourceExtractorBuilder};
use pulp::find::EbookFinder;
use pulp::meta;

fn load_epub(cmd_args: &ArgMatches) -> Result<EpubDoc<BufReader<File>>> {
    let path = cmd_args.value_of("FILE").expect("Must pass FILE");
    EpubDoc::new(path).with_context(|| format!("Failed to read epub from {}", path))
}

fn cmd_describe_toc<R: Read + Seek>(book: EpubDoc<R>, json: bool) -> Result<()> {
    let tr_els = flatten_navpoints(&book.toc);

    if json {
        for toc_el in tr_els {
            let s = serde_json::to_string(&toc_el)?;
            println!("{}", s);
        }
        return Ok(());
    }
    let pad = tr_els
        .iter()
        .map(|el| el.label.len())
        .fold(0, |acc, l| if l > acc { l } else { acc });
    let pad = pad.max(5);
    println!(
        "{}\t{:0width$}\t{}",
        "Play Order",
        "Label",
        "Content",
        width = pad
    );
    for toc_el in tr_els {
        println!(
            "{:<10}\t{:0width$}\t{}",
            toc_el.order,
            toc_el.label,
            toc_el.path_as_string(),
            width = pad
        );
    }
    Ok(())
}

fn main() -> Result<()> {
    let app_input = App::new("pulp")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("0.0.1")
        .about("dump epub contents to plaintext or json")
        .subcommand(
            SubCommand::with_name("toc")
                .about("Print tox.ncx contents")
                .arg(Arg::with_name("FILE").required(true))
                .arg(
                    Arg::with_name("json")
                        .short("j")
                        .help("Output line-delimited JSON")
                        .long("json")
                        .takes_value(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("chapters")
                .about("Print chapters that would be extracted from an ebook")
                .arg(Arg::with_name("FILE").required(true))
                .arg(
                    Arg::with_name("unique")
                        .short("u")
                        .long("unique")
                        .takes_value(false)
                        .help("Print only unique resources"),
                ),
        )
        .subcommand(
            SubCommand::with_name("meta")
                .about("Print ebook metadata and quit")
                .arg(Arg::with_name("FILE").required(true)),
        )
        .subcommand(
            SubCommand::with_name("json")
                .about("Print content as JSON")
                .arg(Arg::with_name("FILE").required(true)),
        )
        .subcommand(
            SubCommand::with_name("batch")
                .about("Batch-process books")
                .arg(
                    Arg::with_name("dir")
                        .short("d")
                        .long("dir")
                        .takes_value(true)
                        .help("Directory containing epub files"),
                )
                .arg(
                    Arg::with_name("glob")
                        .short("g")
                        .long("glob")
                        .takes_value(true)
                        .help("Glob expression"),
                )
                .arg(
                    Arg::with_name("dry-run")
                        .short("d")
                        .long("dry-run")
                        .takes_value(false)
                        .help("Print files that would be processed, but do nothing"),
                ),
        )
        .get_matches();

    match app_input.subcommand() {
        ("meta", Some(cmd_args)) => {
            let book = load_epub(cmd_args)?;
            let map = meta::meta_vars_from_metadata(&book);

            let stdout = ::std::io::stdout();
            let handle = stdout.lock();
            serde_json::to_writer(handle, &map)
                .with_context(|| format!("Failed to convert metadata to JSON!"))?;
        }
        ("json", Some(cmd_args)) => {
            let mut book = load_epub(cmd_args)?;
            let simple_book = transform_simple(&mut book)?;

            let stdout = ::std::io::stdout();
            let handle = stdout.lock();
            serde_json::to_writer(handle, &simple_book)
                .with_context(|| format!("Failed to convert book data to JSON!"))?;
        }
        ("chapters", Some(cmd_args)) => {
            let mut book = load_epub(cmd_args)?;
            let ex = ResourceExtractorBuilder::default().build()?;
            let mut matches = ex.extract(&mut book)?;
            let stdout = ::std::io::stdout();
            let handle = stdout.lock();
            if cmd_args.is_present("unique") {
                matches = matches.into_unique();
            }
            serde_json::to_writer_pretty(handle, &matches)
                .with_context(|| format!("Failed to convert metadata to JSON!"))?;
        }
        ("toc", Some(cmd_args)) => {
            let book = load_epub(cmd_args)?;
            cmd_describe_toc(book, cmd_args.is_present("json"))?;
        }
        ("batch", Some(cmd_args)) => {
            let mut file_list: Vec<PathBuf> = vec![];
            if let Some(dir) = cmd_args.value_of("dir") {
                file_list.extend(EbookFinder::new_from_path(dir)?);
            };
            file_list.sort();
            if cmd_args.is_present("dry-run") {
                for path in file_list {
                    println!("{}", path.display());
                }
            }
        }
        _ => {}
    }
    Ok(())
}
