mod config;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::{self, File};

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

fn main() -> Result<(), Box<dyn Error>> {
    let cmd = Command::from_args();

    let manifest = fs::read_to_string(&cmd.filename)?;

    let mut config = cmd
        .config
        .map(|path| Config::from_reader(File::open(path)?))
        .transpose()?
        .unwrap_or_default();
    config.insert("Env".to_string(), env::vars().collect::<HashMap<_, _>>());

    let result = gtmpl::template(&manifest, config)?;
    print!("{}", result);

    Ok(())
}
