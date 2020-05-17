use std::collections::HashMap;
use std::io;

use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    InvalidYaml(#[from] yaml_rust::ScanError),

    #[error("the file is empty")]
    NoDocuments,

    #[error("the file contains {0} YAML documents but only 1 is allowed")]
    MultipleDocuments(usize),

    #[error("the YAML root is not a mapping")]
    RootNotMapping(Yaml),

    #[error("the YAML contains a non-string key: {0:?}")]
    NonStringKey(Yaml),

    #[error("the YAML contains a value that can't be used in a template: {0:?}")]
    InvalidValue(Yaml),
}

#[derive(Default)]
pub struct Config {
    value: HashMap<String, gtmpl::Value>,
}

impl Config {
    pub fn from_reader(mut reader: impl io::Read) -> Result<Self, Error> {
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let mut yaml = YamlLoader::load_from_str(&contents)?;
        let yaml = match yaml.len() {
            1 => yaml.remove(0),
            0 => return Err(Error::NoDocuments),
            len => return Err(Error::MultipleDocuments(len)),
        };

        match yaml {
            Yaml::Hash(_) => {}
            _ => return Err(Error::RootNotMapping(yaml)),
        }

        let value = match yaml_to_gtmpl(yaml)? {
            gtmpl::Value::Object(value) => value,
            _ => panic!("YAML hash became non-gmpl Value"),
        };

        Ok(Config { value })
    }

    pub fn insert(&mut self, key: String, value: impl Into<gtmpl::Value>) {
        self.value.insert(key, value.into());
    }
}

impl From<Config> for gtmpl::Value {
    fn from(config: Config) -> Self {
        gtmpl::Value::Object(config.value)
    }
}

/// Convert a [`yaml_rust::Yaml`] value into a [`gtmpl::Value`].
///
/// The implementation is based on the `into_*` methods on `yaml_rust::Yaml`, rather than matching
/// to convert between the enums. This is due to some non-trivial incompatibilities such as
/// `Yaml::Real` storing its value as a `String` and the resolution of `Yaml::Alias` values.
fn yaml_to_gtmpl(input: Yaml) -> Result<gtmpl::Value, Error> {
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
            .try_for_each::<_, Result<_, Error>>(|(key, value)| {
                let key = match key.as_str() {
                    Some(_) => key.into_string().unwrap(),
                    None => return Err(Error::NonStringKey(key)),
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
            .try_for_each::<_, Result<_, Error>>(|value| {
                output.push(yaml_to_gtmpl(value)?);
                Ok(())
            })?;
        Ok(Array(output))
    } else {
        Err(Error::InvalidValue(input))
    }
}
