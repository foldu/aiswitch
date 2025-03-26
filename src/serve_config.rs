use std::{collections::HashMap, net::SocketAddr, path::Path};

use eyre::Context;

use crate::runner::Runner;

#[derive(serde::Deserialize)]
pub struct Config {
    pub default: String,
    pub default_model: Option<String>,
    pub addr: SocketAddr,
    pub runners: HashMap<String, Runner>,
}

fn deserialize_config(content: &str) -> Result<Config, eyre::Error> {
    toml::from_str(content).context("Failed deserializing config")
}

pub async fn load(path: impl AsRef<Path>) -> Result<Config, eyre::Error> {
    let path = path.as_ref();
    let data = tokio::fs::read_to_string(path)
        .await
        .context("Failed reading config")?;
    deserialize_config(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_config_deserializes() {
        deserialize_config(include_str!("../example_config.toml")).unwrap();
    }
}
