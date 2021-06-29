use epub::doc::EpubDoc;
use regex::Regex;
use std::io::{Read, Seek};

use crate::error::Error;

const FRONT_MATTER: [&str; 14] = [
    "cover-image",
    "cover",
    "toc",
    "title",
    "about_book",
    "brief-toc",
    r"bm\d+",
    "dedication",
    "copyright",
    "authorsnote",
    "family_chart",
    "map",
    "picture_section",
    "dedication",
];

const END_MATTER: [&str; 5] = ["about(?:the)?author", "endpage", r"ata\d+", "atb", "brand"];

const RESOURCE_IGNORE: [&str; 5] = [
    r"About(?:The|_)(?:Book|Author)",
    r"Also_?By",
    r"(?:Book)?TitlePage",
    r"Copyright",
    r"Contents",
];

/// Get chapters from the spine.
pub fn get_chapters<R: Read + Seek>(
    book: &mut EpubDoc<R>,
) -> Result<Vec<(String, Vec<u8>)>, Error> {
    let rx_str = format!(r"{}|{}", FRONT_MATTER.join(r"|"), END_MATTER.join(r"|"));
    let ignore = Regex::new(&rx_str).unwrap();

    let rx_str = RESOURCE_IGNORE.join(r"|");
    let res_ignore = Regex::new(&rx_str).unwrap();

    let chaps = book
        .spine
        .iter()
        .filter(|res| !ignore.is_match(res))
        .cloned()
        .filter(|id| match &book.resources.get(id) {
            Some((path_buf, _mime)) => match path_buf.to_str() {
                Some(path) => !res_ignore.is_match(path),
                None => false,
            },
            None => false,
        })
        .collect::<Vec<_>>();
    let chaps = chaps
        .into_iter()
        .map(|res| (res.clone(), book.get_resource(&res)))
        .filter(|(_res, vec)| vec.is_ok())
        .map(|(res, vec)| (res, vec.unwrap()))
        .collect();
    Ok(chaps)
}
