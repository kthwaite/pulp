#[derive(Debug)]
pub enum Error {
    NoChapters,
    IoError(std::io::Error),
    RegexError(regex::Error),
}

impl std::error::Error for Error { }
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::NoChapters => write!(f, "No chapters found!"),
            Error::RegexError(err) => write!(f, "{}", err),
            Error::IoError(err) => write!(f, "{}", err),
        }
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}
impl std::convert::From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::RegexError(err)
    }
}
