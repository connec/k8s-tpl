use std::collections::HashMap;
use std::error::Error;
use std::io;

use yaml_rust::{Yaml, YamlLoader};

#[derive(Default)]
pub struct Config {
    value: HashMap<String, gtmpl::Value>,
}

impl Config {
    pub fn from_reader(mut reader: impl io::Read) -> Result<Self, Box<dyn Error>> {
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let mut yaml = YamlLoader::load_from_str(&contents)?;
        let yaml = match yaml.len() {
            1 => yaml.remove(0),
            0 => return Err("The file is empty".into()),
            len => {
                return Err(format!(
                    "The file contains multiple ({}) YAML documents but only one is allowed",
                    len
                )
                .into())
            }
        };

        match yaml {
            Yaml::Hash(_) => {}
            _ => return Err("The YAML root is not a mapping".into()),
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
                let key = match key.as_str() {
                    Some(_) => key.into_string().unwrap(),
                    None => {
                        return Err(format!("The YAML contains a non-string key: {:?}", key).into())
                    }
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
        Err(format!(
            "The YAML contains a value that can't be used in a template: {:?}",
            input
        )
        .into())
    }
}
