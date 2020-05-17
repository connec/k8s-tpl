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
            _ => panic!("YAML hash became non-gmpl Value::Object"),
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

fn yaml_to_gtmpl(input: Yaml) -> Result<gtmpl::Value, Error> {
    match input {
        Yaml::Array(value) => value
            .into_iter()
            .try_fold(Vec::new(), |mut result, item| {
                result.push(yaml_to_gtmpl(item)?);
                Ok(result)
            })
            .map(gtmpl::Value::from),
        Yaml::Boolean(value) => Ok(value.into()),
        Yaml::Hash(value) => value
            .into_iter()
            .try_fold(HashMap::new(), |mut result, (key, item)| {
                let key = match key {
                    Yaml::String(key) => key,
                    key => return Err(Error::NonStringKey(key)),
                };
                result.insert(key, yaml_to_gtmpl(item)?);
                Ok(result)
            })
            .map(gtmpl::Value::Object),
        Yaml::Integer(value) => Ok(value.into()),
        value @ Yaml::Real(_) => Ok(value.into_f64().unwrap().into()),
        Yaml::String(value) => Ok(value.into()),
        value => Err(Error::InvalidValue(value)),
    }
}
