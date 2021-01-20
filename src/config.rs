use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
// TODO: update update lock inf with config

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = {
        if let Some(file) = fs::read("config.json").ok() {
            let conf = serde_json::from_slice(&file).ok();
            if conf.is_some() {
                return Config { ..conf.unwrap() };
            }
        }
        println!("Config file not found. Creating now");
        let conf = Config {
            port: 8080,
            mods: HashMap::new(),
        };
        if let Ok(pretty_string) = serde_json::to_string_pretty(&conf) {
            if let Err(e) = fs::write("config.json", pretty_string) {
                println!("Error creating config: {}", e);
            }
        }
        conf
    };

    pub static ref CONFIG_LOCK: Mutex<ConfigLock> = {
        if let Some(file) = fs::read("config-lock.json").ok() {
            let conf = serde_json::from_slice(&file).ok();
            if conf.is_some() {
                return Mutex::new(ConfigLock { ..conf.unwrap() });
            }
        }
        println!("Config-lock file not found. Creating now");
        Mutex::new(ConfigLock {
            port: 8080,
            installer_version: String::new(),
            loader_version: String::new(),
            game_version: String::new(),
            mods: HashMap::new(),
        })
    };
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub mods: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigLock {
    pub port: u16,
    pub installer_version: String,
    pub loader_version: String,
    pub game_version: String,
    pub mods: HashMap<String, [u32; 2]>,
}

impl ConfigLock {
    pub fn is_new(&self, new_id: u32) -> bool {
        !self.mods.values().any(|info| info[0] == new_id)
    }

    pub fn is_same_version(&self, mod_name: &str, file_id: u32) -> bool {
        self.mods.get(mod_name).unwrap()[1].eq(&file_id)
    }

    pub fn update_file_id(&self, mod_name: &str, mod_id: u32, new_file_id: u32) {
        self.mods.get(mod_name).insert(&[mod_id, new_file_id]);
        if let Err(e) = fs::write(
            "config-lock.json",
            serde_json::to_string_pretty(&self).unwrap(),
        ) {
            println!("Error updating mod: {} in config-lock: {}", mod_name, e);
        }
    }

    pub fn new_mod(&mut self, mod_name: &str, mod_id: u32, file_id: u32) {
        self.mods.insert(mod_name.to_string(), [mod_id, file_id]);

        if let Err(e) = fs::write(
            "config-lock.json",
            serde_json::to_string_pretty(&self).unwrap().as_bytes(),
        ) {
            println!("Error adding mod: {} to config-lock: {}", mod_name, e);
        }
    }

    pub fn update_installer_version(&mut self, new_version: String) {
        self.installer_version = new_version.to_string();

        if let Err(e) = fs::write(
            "config-lock.json",
            serde_json::to_string_pretty(&self).unwrap().as_bytes(),
        ) {
            println!("Error updating installer version in config-lock: {}", e);
        }
    }

    pub fn update_loader_version(&mut self, new_version: String) {
        self.loader_version = new_version;

        if let Err(e) = fs::write(
            "config-lock.json",
            serde_json::to_string_pretty(&self).unwrap().as_bytes(),
        ) {
            println!("Error updating loader version in config-lock: {}", e);
        }
    }

    pub fn update_game_version(&mut self, new_version: String) {
        self.game_version = new_version;

        if let Err(e) = fs::write(
            "config-lock.json",
            serde_json::to_string_pretty(&self).unwrap().as_bytes(),
        ) {
            println!("Error updating game version in config-lock: {}", e);
        }
    }
}
