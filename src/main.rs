mod config;

use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io;

use structopt::StructOpt;

use self::config::Config;

/// Templatisation for Kubernetes manifests
#[derive(Debug, StructOpt)]
struct Command {
    /// The path to a Kubernetes manifest template.
    #[structopt(long, short)]
    filename: String,

    /// The path to a configuration YAML file.
    ///
    /// The YAML file should contain a single document with a mapping at the root. All mappings in
    /// the document must have only string keys.
    #[structopt(long, short)]
    config: Option<String>,
}

enum Error {
    Io(String, io::Error),
    Config(String, config::Error),
    Template(String, String),
}

impl Error {
    fn from_io_error(path: &str, error: io::Error) -> Self {
        Error::Io(path.to_string(), error)
    }

    fn from_config_error(path: &str, error: config::Error) -> Self {
        Error::Config(path.to_string(), error)
    }

    fn from_template_error(path: &str, error: String) -> Self {
        Error::Template(path.to_string(), error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(path, source) => write!(f, "Error reading file {}: {}", path, source),
            Error::Config(path, source) => {
                write!(f, "Failed to load configuration file {}: {}", path, source)
            }
            Error::Template(path, message) => {
                write!(f, "Failed to process template {}: {}", path, message)
            }
        }
    }
}

fn main() {
    if let Err(error) = _main() {
        eprintln!("Error:\n  {}", error);
    }
}

fn _main() -> Result<(), Error> {
    let cmd = Command::from_args();

    let manifest = fs::read_to_string(&cmd.filename)
        .map_err(|error| Error::from_io_error(&cmd.filename, error))?;

    let mut config = match &cmd.config {
        Some(path) => Config::from_reader(
            File::open(path).map_err(|error| Error::from_io_error(&path, error))?,
        )
        .map_err(|error| Error::from_config_error(path, error))?,
        None => Config::default(),
    };
    config.insert("Env".to_string(), env::vars().collect::<HashMap<_, _>>());

    let result = gtmpl::template(&manifest, config)
        .map_err(|error| Error::from_template_error(&cmd.filename, error))?;
    print!("{}", result);

    Ok(())
}
