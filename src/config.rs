use std::fs;
use std::sync::Mutex;
use super::mods::MCMod;

// TODO: update update lock inf with config

pub fn create_new_config() -> Config {
    let def_conf = Config {
        port: 8080,
        curseforge_mods: Vec::new(),
        modrinth_mods: Vec::new(),
    };

        println!("config.json might not exist. Creating now");
        match serde_json::to_string_pretty(&def_conf) {
            Ok(pretty_string) => {
                match fs::write("config.json", pretty_string) {
                    Ok(_) => println!("Created config.json"),
                    Err(e) => println!("Error creating config.json: {}", e),
                }
            },
            Err(e) => println!("Error parsing default config: {}", e)
        }
        def_conf
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = {
            match fs::read("config.json") {
                Ok(file) => match serde_json::from_slice::<Config>(&file) {
                    Ok(conf) => conf,
                    _ => create_new_config(),
                },
                _ => create_new_config(),
            }
        };


    pub static ref CONFIG_LOCK: Mutex<ConfigLock> = {
        let def_config_lock = ConfigLock {
            port: 8080,
            installer_version: String::new(),
            loader_version: String::new(),
            game_version: String::new(),
            curseforge_mods: Vec::new(),
            modrinth_mods: Vec::new(),
        };

        match fs::read("config-lock.json") {
            Ok(file) => match serde_json::from_slice(&file) {
                Ok(conf) => Mutex::new(conf),
                _ => Mutex::new(def_config_lock),
            },
            _ => Mutex::new(def_config_lock),
        }
    };
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub curseforge_mods: Vec<MCMod>,
    pub modrinth_mods: Vec<MCMod>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigLock {
    pub port: u16,
    pub installer_version: String,
    pub loader_version: String,
    pub game_version: String,
    pub curseforge_mods: Vec<MCMod>,
    pub modrinth_mods: Vec<MCMod>,
}

impl ConfigLock {
    pub fn is_new(&self, possibly_new_mod: &MCMod) -> bool {
        match possibly_new_mod.platform {
            Platform::Curseforge => !self.curseforge_mods.iter().any(|old_mod| old_mod.mod_name == possibly_new_mod.mod_name),
            Platform::Modrinth => !self.modrinth_mods.iter().any(|old_mod| old_mod.mod_name == possibly_new_mod.mod_name)
        }
    }

    pub fn is_same_version(&self, mc_mod: &MCMod) -> bool {
        match mc_mod.platform {
            Platform::Curseforge => self.curseforge_mods.iter().any(|mod_to_compare| mod_to_compare.file_id.eq(&mc_mod.file_id)),
            Platform::Modrinth => self.modrinth_mods.iter().any(|mod_to_compare| mod_to_compare.file_id.eq(&mc_mod.file_id)),
        }
    }

    pub fn update_file_id(&self, old_file_id: u64, new_file_id: u64, platform: Platform) {
        match platform {
            Platform::Curseforge => self.curseforge_mods.iter().for_each(|mc_mod| if mc_mod.file_id == Some(old_file_id) { MCMod::copy(mc_mod).file_id = Some(new_file_id)} ),
            Platform::Modrinth => self.modrinth_mods.iter().for_each(|mc_mod| if mc_mod.file_id == Some(old_file_id) { MCMod::copy(mc_mod).file_id = Some(new_file_id)} ),
        };
        
        self.update_file();
    }

    pub fn new_mod(&mut self, mc_mod: MCMod) {
        match mc_mod.platform {
            Platform::Curseforge => self.curseforge_mods.push(mc_mod),
            Platform::Modrinth => self.modrinth_mods.push(mc_mod)
        };

        self.update_file();
    }

    pub fn update_installer_version(&mut self, new_version: String) {
        self.installer_version = new_version.to_string();

        self.update_file();
    }

    pub fn update_loader_version(&mut self, new_version: String) {
        self.loader_version = new_version;

        self.update_file();
    }

    pub fn update_game_version(&mut self, new_version: String) {
        self.game_version = new_version;

        self.update_file();
    }

    fn update_file(&self) -> () {
        match serde_json::to_string_pretty(self) {
            Ok(pretty_string) => if let Err(e) = fs::write("config-lock.json", &pretty_string) {
                println!("Error writing to config-lock.json: {}", e);
            },
            Err(_) => println!("Error parsing config-lock"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Platform {
    Modrinth,
    Curseforge,
}