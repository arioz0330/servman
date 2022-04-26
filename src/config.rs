use std::fs;
use async_mutex::Mutex;
extern crate serde_xml_rs;

// TODO: add option for console to terminal

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  pub port: u16,
  pub installer_version: String,
  pub loader_version: String,
  pub game_version: String,
}

impl Config {
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
    match serde_xml_rs::to_string(self) {
      Ok(string) => {
        if let Err(e) = fs::write("config.xml", &string) {
          println!("Error writing to config.xml: {}", e);
        }
      }
      Err(_) => println!("Error parsing config"),
    }
  }
}

pub fn create_new_config() -> Config {
  let def_config = Config {
    port: 8080,
    installer_version: String::new(),
    loader_version: String::new(),
    game_version: String::new(),
  };

  println!("config.xml does not exist. Creating now");
  match serde_xml_rs::to_string(&def_config) {
    Ok(string) => match fs::write("config.xml", string) {
      Ok(_) => println!("Created config.xml"),
      Err(e) => println!("Error creating config.xml: {}", e),
    },
    Err(e) => println!("Error parsing default config: {:?}", e),
  }
  def_config
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Mutex<Config> = {
            match fs::read_to_string("config.xml") {
                Ok(file) => match serde_xml_rs::from_str::<Config>(&file) {
                    Ok(conf) => Mutex::new(conf),
                    _ => Mutex::new(create_new_config()),
                },
                _ => Mutex::new(create_new_config()),
            }
        };
}