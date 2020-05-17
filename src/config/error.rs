use std::io;

use yaml_rust::Yaml;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidYaml(yaml_rust::ScanError),
    NoDocuments,
    MultipleDocuments(usize),
    RootNotMapping(Yaml),
    NonStringKey(Yaml),
    InvalidValue(Yaml),
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<yaml_rust::ScanError> for Error {
    fn from(error: yaml_rust::ScanError) -> Self {
        Self::InvalidYaml(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Error::*;
        match self {
            Io(error) => write!(f, "{}", error),
            InvalidYaml(error) => write!(f, "{}", error),
            NoDocuments => write!(f, "The file is empty"),
            MultipleDocuments(n) => write!(
                f,
                "The file contains {} YAML documents but only 1 is allowed",
                n
            ),
            RootNotMapping(_) => write!(f, "The YAML root is not a mapping"),
            NonStringKey(key) => write!(f, "The YAML contains a non-string key: {:?}", key),
            InvalidValue(value) => write!(
                f,
                "The YAML contains a value that can't be used in a template: {:?}",
                value
            ),
        }
    }
}
