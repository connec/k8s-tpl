mod config;

use std::collections::HashMap;
use std::env;
use std::fs::{self, File};

use anyhow::{Context, Error, Result};
use structopt::StructOpt;

use self::config::Config;

/// Templatisation for Kubernetes manifests
#[derive(Debug, StructOpt)]
struct Command {
    /// The path to a configuration YAML file.
    ///
    /// The YAML file should contain a single document with a mapping at the root. All mappings in
    /// the document must have only string keys.
    #[structopt(long, short)]
    config: Option<String>,

    /// The path to a Kubernetes manifest template.
    filename: String,
}

fn main() -> Result<()> {
    let cmd = Command::from_args();

    let manifest = fs::read_to_string(&cmd.filename)
        .with_context(|| format!("failed to read {}", cmd.filename))?;

    let config = get_config(cmd.config.as_deref())?;

    let result = gtmpl::template(&manifest, config)
        .map_err(Error::msg)
        .with_context(|| format!("failed to render {}", cmd.filename))?;
    print!("{}", result);

    Ok(())
}

fn get_config(path: Option<&str>) -> Result<Config> {
    fn load_config(path: &str) -> Result<Config> {
        File::open(path)
            .map_err(Error::new)
            .and_then(|file| Config::from_reader(file).map_err(Error::new))
            .with_context(|| format!("failed to read config {}", &path))
    }

    let mut config = path.map(load_config).transpose()?.unwrap_or_default();
    config.insert("Env".to_string(), env::vars().collect::<HashMap<_, _>>());

    Ok(config)
}
