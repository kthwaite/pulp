use epub::doc::EpubDoc;
use failure::Error;

const DEFAULT_FIELDS : [&'static str; 11] = [
    "title",
    "creator",
    "date",
    "identifier",
    "source",
    "publisher",
    "imprint",
    "language",
    "format",
    "fixed-layout",
    "type"
];

pub fn list_meta(book: &mut EpubDoc, fields: Option<&[String]>, _with_fieldnames: bool) -> Result<(), Error> {
    let fields : Vec<String> = match fields {
        Some(fls) => {
            fls.into_iter()
                  .filter(|&field| book.metadata.contains_key(field))
                  .cloned()
                  .collect()
        },
        None => {
            DEFAULT_FIELDS.iter()
                .filter(|&field| book.metadata.contains_key(*field))
                .map(|&field| String::from(field))
                .collect()
        }
    };
    for field in fields {
        match book.metadata.get(&field) {
            Some(value) => println!("{} - {}", field, value.join(", ")),
            None => println!("{} - UNKNOWN", field)
        }
    }
    Ok(())
}

