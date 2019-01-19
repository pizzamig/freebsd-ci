use failure::Error;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) tokens: Tokens,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Tokens {
    pub(crate) github: String,
}

pub(crate) fn get_config(path: &Path) -> Result<Config, Error> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut buf: Vec<u8> = Vec::new();
    let _len = reader.read_to_end(&mut buf)?;
    let config: Config = toml::from_slice(&buf)?;
    Ok(config)
}
