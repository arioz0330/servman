use std::fs;
use async_mutex::Mutex;

// TODO: add option for console to terminal

pub fn create_new_config() -> Config {
  let def_config = Config {
        port: 8080,
      };

  println!("config.json does not exist. Creating now");

  match serde_json::to_string_pretty(&def_config) {
    Ok(pretty_string) => match fs::write("config.json", pretty_string) {
      Ok(_) => println!("Created config.json"),
      Err(e) => println!("Error creating config.json: {}", e),
    },
    Err(e) => println!("Error parsing default config: {}", e),
  }
  def_config
}

pub fn create_new_config_lock(config_lock: ConfigLock) -> ConfigLock {
  println!("config-lock.json does not exist. Creating now");

  match serde_json::to_string_pretty(&config_lock) {
    Ok(pretty_string) => match fs::write("config-lock.json", pretty_string) {
      Ok(_) => println!("Created config.json"),
      Err(e) => println!("Error creating config.json: {}", e),
    },
    Err(e) => println!("Error parsing default config: {}", e),
  }
  config_lock
}

#[derive(Serialize, Deserialize)]
pub struct Config {
  pub port: u16,
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
        };

        match fs::read("config-lock.json") {
            Ok(file) => match serde_json::from_slice(&file) {
                Ok(conf) => {
                  let mut typed_conf: ConfigLock = conf;
                  if CONFIG.port != typed_conf.port {
                    typed_conf.port = CONFIG.port;
                    typed_conf.update_file();
                  }
                  Mutex::new(typed_conf)
                },
                _ => Mutex::new(def_config_lock),
            },
            _ => {
              let new_config_lock: ConfigLock = ConfigLock { port: CONFIG.port, ..def_config_lock };
              Mutex::new(create_new_config_lock(new_config_lock))
            },
        }
    };
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigLock {
  pub port: u16,
  pub installer_version: String,
  pub loader_version: String,
  pub game_version: String,
}

impl ConfigLock {
  pub fn update_installer_version(&mut self, new_version: String) {
    self.installer_version = new_version;

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

  fn update_file(&self) {
    match serde_json::to_string_pretty(self) {
      Ok(pretty_string) => {
        if let Err(e) = fs::write("config-lock.json", &pretty_string) {
          println!("Error writing to config-lock.json: {}", e);
        }
      }
      Err(_) => println!("Error parsing config-lock"),
    }
  }
}