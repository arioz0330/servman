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

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigLock {
    pub port: u16,
    pub mods: HashMap<String, [u32; 2]>,
}

impl ConfigLock {
    pub fn new() -> Self {
        match fs::read("config-lock.json") {
            Ok(file) => {
                match serde_json::from_slice(&file) {
                    Ok(conflock) => Self { ..conflock },
                    Err(e) => {
                        println!("Error opening config-lock: {}", e);
                        let port = 8080;
                        let mods: HashMap<String, [u32; 2]> = HashMap::new();
                        Self { port, mods }
                    }
                }
            },
            Err(e) => {
                println!("Error opening config-lock: {}", e);
                let port = 8080;
                let mods: HashMap<String, [u32; 2]> = HashMap::new();
                Self { port, mods }
            },
        }
    }

    pub fn is_new(self, new_id: u32) -> bool {
        !self.mods.values().any(|info| info[0] == new_id)
    }

    pub fn is_same_version(self, mod_name: &String, file_id: u32) -> bool {
            self.mods.get(mod_name).unwrap()[1].eq(&file_id)
    }

    pub fn update_file_id(self, mod_name:  &String, mod_id: u32, new_file_id: u32) -> () {
        self.mods.get(mod_name).insert(&mut [mod_id, new_file_id]);
        match fs::write("config-lock.json", serde_json::to_string_pretty(&self).unwrap()) {
            Ok(()) => println!("Updated mod: {} in config-lock", mod_name),
            Err(e) => println!("Error updating mod: {} in config-lock: {}", mod_name, e),
        }
        ()
    }

    pub fn new_mod(mut self, mod_name: &String, mod_id: u32, file_id: u32) {
        let _ = self.mods.insert(mod_name.clone(), [mod_id, file_id]);
        match fs::write("config-lock.json", serde_json::to_string_pretty(&self).unwrap().as_bytes()) {
            Ok(()) => println!("Added mod: {} to config-lock", mod_name),
            Err(e) => println!("Error adding mod: {} to config-lock: {}", mod_name, e),
        }
    }
}