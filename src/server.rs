extern crate serde_xml_rs;

use std::{fmt, fs, thread};
use std::{path::Path, sync::Arc};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, Command, Stdio};
use std::sync::RwLock;

use super::config;

type Result<T> = std::result::Result<T, ServerErrors>;

const FABRIC_INSTALLER: &str = "https://maven.fabricmc.net/net/fabricmc/fabric-installer";

#[derive(Deserialize, Debug)]
struct Metadata {
  versioning: Versioning,
}

#[derive(Deserialize, Debug)]
struct Versioning {
  latest: Latest,
}

#[derive(Deserialize, Debug)]
struct Latest {
  #[serde(rename = "$value")]
  data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LatestFile {
  #[serde(rename = "gameVersion")]
  game_version: String,
  #[serde(rename = "projectFileId")]
  project_file_id: u64,
  #[serde(rename = "projectFileName")]
  project_file_name: String,
}

#[derive(Debug)]
pub enum ServerErrors {
  IoError(std::io::Error),
  ServerOffline(),
  ServerAlreadyOnline(),
  ServerFilesMissing(),
  ServerAlreadyExists(),
  ThreadError(String),
  ServerProcessExited(),
  ServerStillStarting(),
  NetworkError(),
  FileError(String),
}

impl std::error::Error for ServerErrors {
  fn description(&self) -> &str {
    match *self {
      ServerErrors::IoError(_) => "IOError",
      ServerErrors::ServerFilesMissing() => "MissingServer",
      ServerErrors::ServerOffline() => "ServerOffline",
      ServerErrors::ServerAlreadyExists() => "ServerAlreadyExists",
      ServerErrors::ThreadError(_) => "ThreadError",
      ServerErrors::ServerProcessExited() => "ServerProcessExited",
      ServerErrors::ServerAlreadyOnline() => "ServerAlreadyOnline",
      ServerErrors::ServerStillStarting() => "ServerStillStarting",
      ServerErrors::NetworkError() => "NetworkError",
      ServerErrors::FileError(_) => "FileError"
    }
  }
}

impl fmt::Display for ServerErrors {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      ServerErrors::IoError(ref a) => write!(f, "Io error: {}", a),
      ServerErrors::ServerFilesMissing() => write!(f, "Server files not found"),
      ServerErrors::ServerOffline() => write!(f, "Server offline while called."),
      ServerErrors::ServerAlreadyExists() => write!(f, "Server files already exist"),
      ServerErrors::ThreadError(ref a) => write!(f, "Error while creating {} thread for server", a),
      ServerErrors::ServerProcessExited() => {
        write!(f, "Server processes needed but has unexpectedly exited.")
      }
      ServerErrors::ServerAlreadyOnline() => write!(f, "Attempted to start server whilst online"),
      ServerErrors::ServerStillStarting() => {
        write!(f, "Attempted to stop server whilst mid-loading")
      },
      ServerErrors::NetworkError() => write!(f, "Error getting files from network"),
      ServerErrors::FileError(ref a) => write!(f, "Error parsing or saving a file(s): {}", a),
    }
  }
}

impl From<std::io::Error> for ServerErrors {
  fn from(e: std::io::Error) -> Self {
    ServerErrors::IoError(e)
  }
}

/// Controls the creation and deleting of server, and whether they are currently active.
pub struct Manager {
  server: Option<Instance>,
  jar_name: &'static str,
}

impl Default for Manager {
  fn default() -> Self {
    Self::new()
  }
}

impl Manager {
  /// Creates a new server manager
  /// # Examples
  /// ```
  ///   let manager = server::Manager::new();
  /// ```
  /// # Remarks
  /// The version_folder should be a folder that contains folders that are named the same as the MC server files they contain.
  pub fn new() -> Manager {
    Manager {
      server: None,
      jar_name: "fabric-server-launch.jar",
    }
  }

  /// Creates a new MC server folder under the `server_files_folder`
  /// # Examples
  /// ```
  /// let manager = servman::Manager::new();
  /// manager.create();
  /// ```
  /// # Remarks
  /// Returns a result that contains the version of the server
  pub async fn create(&mut self) -> Result<()> {
    if self.exists() {
      return Err(ServerErrors::ServerAlreadyExists());
    }

    match fs::create_dir("server") {
      Ok(()) => println!("Server directory created"),
      Err(e) => println!("Error creating server directory: {}", e),
    };

    self.update().await?;

    Ok(())
  }

  /// Checks if server files exist
  pub fn exists(&mut self) -> bool {
    Path::new(&format!("server/{}", self.jar_name)).exists()
  }

  /// Launches a server
  pub fn start(&mut self) -> Result<u32> {
    if !self.exists() {
      return Err(ServerErrors::ServerFilesMissing());
    }

    if self.server.is_some() {
      Err(ServerErrors::ServerAlreadyOnline())
    } else {
      let mut command = Command::new("java");
      command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).args(&["-Xmx4G", "-Xms4G", "-jar", self.jar_name, "nogui"]).current_dir("server");
      let mut serv_inst = Instance {
        server_process: command.spawn()?,
        stdout_join: None,
        starting: Arc::new(RwLock::new(true)),
      };

      let stdout = match serv_inst.server_process.stdout.take() {
        Some(e) => e,
        None => return Err(ServerErrors::ThreadError("stdout".to_string())),
      };

      let starting_lock = serv_inst.starting.clone();

      let stdout_thread_handle = thread::spawn(move || {
        BufReader::new(stdout).lines().for_each(|x| {
          let a = x.unwrap();
          println!("{}", &a);
          if a.len() > 38 && &a[33..37] == "Done" {
            *starting_lock.write().unwrap() = false;
          }
        });
      });

      serv_inst.stdout_join = Some(stdout_thread_handle);
      self.server = Some(serv_inst);
      Ok(25565)
    }
  }

  /// Stops a server
  pub fn stop(&mut self) -> Result<()> {
    if let Some(inst) = &mut self.server {
      if !*inst.starting.read().unwrap() {
        inst.stop()?;
        let _ = inst.stdout_join.take().unwrap().join();
        inst.server_process.wait()?;
        self.server = None;
        return Ok(());
      }
      return Err(ServerErrors::ServerStillStarting());
    }
    Err(ServerErrors::ServerOffline())
  }

  /// OPs user with specified name
  pub fn op(&mut self, name: &str) -> Result<()> {
    if let Some(inst) = &mut self.server {
      match inst.send(format!("/op {}", name)) {
        Ok(()) => println!("{} is now op", name),
        Err(e) => println!("Error adding {} to op: {}", name, e),
      };
      return Ok(());
    }
    Err(ServerErrors::ServerOffline())
  }

  /// OPs user with specified name
  pub fn de_op(&mut self, name: &str) -> Result<()> {
    if let Some(inst) = &mut self.server {
      match inst.send(format!("/deop {}", name)) {
        Ok(()) => println!("{} is no longer op", name),
        Err(e) => println!("Error removing {} from op: {}", name, e),
      };
      return Ok(());
    }
    Err(ServerErrors::ServerOffline())
  }

  /// Deletes a server's files
  /// # Remarks
  /// Stops the server if it's currently running
  pub fn delete(&mut self) -> Result<()> {
    self.stop()?;
    if !self.exists() {
      return Err(ServerErrors::ServerFilesMissing());
    }
    fs::remove_dir_all("server")?;
    Ok(())
  }

  /// Changes a server's version
  /// # Remarks
  /// Stops the server if it's currently running
  pub async fn update(&mut self) -> Result<()> {
    if self.server.is_some() {
      match self.stop() {
        Ok(()) => println!("Server stopped"),
        Err(e) => println!("Error stopping server: {}", e),
      }
    }

    use serde_xml_rs::from_str;

    let mut config_lock = config::CONFIG_LOCK.lock().await;

    let fabric: Metadata = match attohttpc::get(&format!("{}/maven-metadata.xml", FABRIC_INSTALLER)).send() {
      Ok(response) => {
        match response.text() {
          Ok(response_as_string) => {
            match from_str(&response_as_string) {
              Ok(metadata_as_xml) => metadata_as_xml,
              Err(_) => return Err(ServerErrors::FileError("metadata".to_string())),
            }
          },
          Err(_) => return Err(ServerErrors::FileError("metadata".to_string())),
        }
      },
      Err(_) => return Err(ServerErrors::NetworkError()),
    };

    let ver = fabric.versioning.latest.data;

    if !config_lock.installer_version.eq(&ver) {
      println!("Updating installer");

      match attohttpc::get(&format!("{0}/{1}/fabric-installer-{1}.jar", FABRIC_INSTALLER, ver)).send() {
        Ok(response) => match response.bytes() {
          Ok(installer_as_bytes) => {
            match fs::File::create("./server/fabric-installer.jar") {
              Ok(mut file) => {
                file.write_all(&installer_as_bytes).unwrap();
                config_lock.update_installer_version(ver);
              },
              Err(_) => return Err(ServerErrors::FileError("installer".to_string())),
            };
          },
          Err(_) => return Err(ServerErrors::FileError("installer".to_string())),
        },
        Err(_) => return Err(ServerErrors::NetworkError()),
      }
    }

    do_eula();

    let mut install = Command::new("java").args(&[
      "-jar",
      "fabric-installer.jar",
      "server",
      "-downloadMinecraft",
    ]).current_dir("server").stdout(Stdio::piped()).spawn()?;

    let a = BufReader::new(install.stdout.take().unwrap()).lines().nth(3).unwrap().unwrap();
    println!("{}", a);
    let left_paren = a.find('(').unwrap();
    let right_paren = a.find(')').unwrap();
    let game_version = &a[left_paren + 1..right_paren].to_string();
    let loader_version = a[25..left_paren].to_string();

    if !config_lock.loader_version.eq(&loader_version) {
      config_lock.update_loader_version(loader_version);
    }

    if !game_version.eq(&config_lock.game_version) {
      config_lock.update_game_version(game_version.to_string());
    }

    Ok(())
  }
}

/// Represents a currently online server.
/// Created by calling [start](struct.Manager.html#method.start) from a [Manager](struct.Manager.html)
#[derive(Debug)]
pub struct Instance {
  pub server_process: Child,
  stdout_join: Option<thread::JoinHandle<()>>,
  starting: Arc<RwLock<bool>>,
}

impl Instance {
  /// Stops the server, killing the server process and the stdin
  /// and stdout threads
  pub fn stop(&mut self) -> Result<()> {
    self.process_check()?;
    self.send(String::from("/stop"))?;
    Ok(())
  }

  /// Checks if the server process is still valid (has not crashed or exited).
  pub fn validity_check(&mut self) -> Result<bool> {
    match self.server_process.try_wait()? {
      Some(_) => Ok(false),
      None => Ok(true),
    }
  }

  fn process_check(&mut self) -> Result<()> {
    match self.validity_check()? {
      true => Ok(()),
      false => Err(ServerErrors::ServerProcessExited()),
    }
  }

  /// Sends a string to the server stdin
  /// # Arguments
  /// * `msg` - A String that contains the message to be sent to the server.
  ///
  /// # Remarks
  /// The message should not contain a trailing newline, as the send method handles it.
  pub fn send(&mut self, msg: String) -> Result<()> {
    self.process_check()?;

    let stdin = match self.server_process.stdin.take() {
      Some(e) => e,
      None => return Err(ServerErrors::ThreadError("stdin".to_string())),
    };

    let mut writer = BufWriter::new(stdin);
    writeln!(writer, "{}", msg)?;
    writer.flush()?;

    Ok(())
  }
}

pub fn do_eula() {
  let _ = fs::OpenOptions::new().write(true).create_new(true).open("server/eula.txt").and_then(|mut file| file.write(b"eula = true"));
}
