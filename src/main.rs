use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::{self};

use structopt::StructOpt;
use yaml_rust::{Yaml, YamlLoader};

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
    config: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cmd = Command::from_args();

    let manifest = load_manifest(&cmd.filename)?;
    let config = load_config(&cmd.config)?;

    let context = yaml_to_gtmpl(config)?;
    let result = gtmpl::template(&manifest, context)?;
    print!("{}", result);

    Ok(())
}

fn load_manifest(path: &str) -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(path)?)
}

fn load_config(path: &str) -> Result<Yaml, Box<dyn Error>> {
    let mut config = YamlLoader::load_from_str(&fs::read_to_string(path)?)?;

    let config = match config.len() {
        0 => return Err(format!("Config file {} is empty", path).into()),
        1 => config.remove(0),
        len => {
            return Err(format!(
                "Config file {} contains multiple ({}) YAML documents – only one is allowed",
                path, len
            )
            .into())
        }
    };

    let mut config = match config {
        Yaml::Hash(config) => config,
        _ => return Err(format!("Config file {} does not contain a mapping", path).into()),
    };

    config.insert(
        Yaml::String("Env".to_string()),
        Yaml::Hash(
            env::vars()
                .map(|(k, v)| (Yaml::String(k), Yaml::String(v)))
                .collect(),
        ),
    );

    Ok(Yaml::Hash(config))
}

/// Convert a [`yaml_rust::Yaml`] value into a [`gtmpl::Value`].
///
/// The implementation is based on the `into_*` methods on `yaml_rust::Yaml`, rather than matching
/// to convert between the enums. This is due to some non-trivial incompatibilities such as
/// `Yaml::Real` storing its value as a `String` and the resolution of `Yaml::Alias` values.
fn yaml_to_gtmpl(input: Yaml) -> Result<gtmpl::Value, Box<dyn Error>> {
    use gtmpl::Value::*;
    if input.as_bool().is_some() {
        Ok(Bool(input.into_bool().unwrap()))
    } else if input.as_f64().is_some() {
        Ok(Number(input.into_f64().unwrap().into()))
    } else if input.as_hash().is_some() {
        let input = input.into_hash().unwrap();
        let mut output = HashMap::with_capacity(input.len());
        input
            .into_iter()
            .try_for_each::<_, Result<_, Box<dyn Error>>>(|(key, value)| {
                let key = match key.into_string() {
                    Some(key) => key,
                    None => return Err("non-string keys are not supported".into()),
                };
                output.insert(key, yaml_to_gtmpl(value)?);
                Ok(())
            })?;
        Ok(Object(output))
    } else if input.as_i64().is_some() {
        Ok(Number(input.into_i64().unwrap().into()))
    } else if input.as_str().is_some() {
        Ok(String(input.into_string().unwrap()))
    } else if input.as_vec().is_some() {
        let input = input.into_vec().unwrap();
        let mut output = Vec::with_capacity(input.len());
        input
            .into_iter()
            .try_for_each::<_, Result<_, Box<dyn Error>>>(|value| {
                output.push(yaml_to_gtmpl(value)?);
                Ok(())
            })?;
        Ok(Array(output))
    } else {
        Err(format!("YAML value cannot be used in Go templates: {:?}", input).into())
    }
}
