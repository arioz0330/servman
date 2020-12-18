use std::collections::HashMap;
use std::fs;

// TODO: this mod will handle all reading and writing of config and data

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub mods: HashMap<String, u32>,
}

impl Config {
    pub fn new() -> Self {
        let conf_file = fs::read("config.json").unwrap();
        let json: Config = serde_json::from_slice(&conf_file).unwrap();
        json
    }
}
