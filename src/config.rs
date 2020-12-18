use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;

// TODO: update update lock inf with config

lazy_static! {
    pub static ref CONFIG: Config = match fs::read("config.json") {
        Ok(file) => match serde_json::from_slice(&file) {
            Ok(conf) => Config { ..conf },
            Err(e) => {
                println!("Error opening config-lock: {}", e);
                let conf = Config {
                    port: 8080,
                    mods: HashMap::new(),
                };
                let _ = fs::write("config.json", serde_json::to_string_pretty(&conf).unwrap().as_bytes());
                conf
            }
        },
        Err(e) => {
            println!("Error opening config-lock: {}", e);
            let conf = Config {
                port: 8080,
                mods: HashMap::new(),
            };
            let _ = fs::write("config.json", serde_json::to_string_pretty(&conf).unwrap().as_bytes());
            conf
        }
    };

    pub static ref CONFIG_LOCK: Mutex<ConfigLock> = match fs::read("config-lock.json") {
        Ok(file) => match serde_json::from_slice(&file) {
            Ok(conflock) => Mutex::new(ConfigLock { ..conflock }),
            Err(e) => {
                println!("Error opening config-lock: {}", e);
                let port = 8080;
                let mods: HashMap<String, [u32; 2]> = HashMap::new();
                Mutex::new(ConfigLock {
                    port,
                    installer_version: String::new(),
                    loader_version: String::new(),
                    game_version: String::new(),
                    mods,
                })
            }
        },
        Err(e) => {
            println!("Error opening config-lock: {}", e);
            let port = 8080;
            let mods: HashMap<String, [u32; 2]> = HashMap::new();
            Mutex::new(ConfigLock {
                port,
                installer_version: String::new(),
                loader_version: String::new(),
                game_version: String::new(),
                mods,
            })
        }
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
        match fs::write(
            "config-lock.json",
            serde_json::to_string_pretty(&self).unwrap(),
        ) {
            Ok(()) => println!("Updated mod: {} in config-lock", mod_name),
            Err(e) => println!("Error updating mod: {} in config-lock: {}", mod_name, e),
        }
    }

    pub fn new_mod(&mut self, mod_name: &str, mod_id: u32, file_id: u32) {
        let _ = self.mods.insert(mod_name.to_string(), [mod_id, file_id]);
        match fs::write(
            "config-lock.json",
            serde_json::to_string_pretty(&self).unwrap().as_bytes(),
        ) {
            Ok(()) => println!("Added mod: {} to config-lock", mod_name),
            Err(e) => println!("Error adding mod: {} to config-lock: {}", mod_name, e),
        }
    }

    pub fn update_installer_version(&mut self, new_version: String) {
        let _ = self.installer_version = new_version.to_string();
        match fs::write(
            "config-lock.json",
            serde_json::to_string_pretty(&self).unwrap().as_bytes(),
        ) {
            Ok(()) => println!("Updated installer to {}", new_version.to_string()),
            Err(e) => println!("Error updating installer version in config-lock: {}", e),
        }
    }

    pub fn update_loader_version(&mut self, new_version: String) {
        let _ = self.loader_version = new_version;
        match fs::write(
            "config-lock.json",
            serde_json::to_string_pretty(&self).unwrap().as_bytes(),
        ) {
            Ok(()) => println!("Updated loader to {}", self.loader_version),
            Err(e) => println!("Error updating loader version in config-lock: {}", e),
        }
    }

    pub fn update_game_version(&mut self, new_version: String) {
        let _ = self.game_version = new_version;
        match fs::write(
            "config-lock.json",
            serde_json::to_string_pretty(&self).unwrap().as_bytes(),
        ) {
            Ok(()) => println!(
                "Updated game version to {} in config file",
                self.loader_version
            ),
            Err(e) => println!("Error updating game version in config-lock: {}", e),
        }
    }
}
