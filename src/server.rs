use super::{config, mods::MCMod};
use std::sync::RwLock;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, Command, Stdio};
use std::{fmt, fs, thread};
use std::{path::Path, sync::Arc};

type Result<T> = std::result::Result<T, Error>;

extern crate serde_xml_rs;

const FABRIC_INSTALLER: &str = "https://maven.fabricmc.net/net/fabricmc/fabric-installer";
const MOD_INFO: &str = "https://addons-ecs.forgesvc.net/api/v2/addon/";

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
struct ModInfo {
    #[serde(rename = "gameVersionLatestFiles")]
    game_version_latest_files: Vec<LatestFile>,
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
pub enum Error {
    IoError(std::io::Error),
    ServerOffline(),
    ServerAlreadyOnline(),
    ServerFilesMissing(),
    ServerAlreadyExists(),
    ThreadError(String),
    ServerProcessExited(),
    ServerStillStarting(),
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(_) => "IOError",
            Error::ServerFilesMissing() => "MissingServer",
            Error::ServerOffline() => "ServerOffline",
            Error::ServerAlreadyExists() => "ServerAlreadyExists",
            Error::ThreadError(_) => "ThreadError",
            Error::ServerProcessExited() => "ServerProcessExited",
            Error::ServerAlreadyOnline() => "ServerAlreadyOnline",
            Error::ServerStillStarting() => "ServerStillStarting",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IoError(ref a) => write!(f, "Io error: {}", a),
            Error::ServerFilesMissing() => write!(f, "Server files not found"),
            Error::ServerOffline() => write!(f, "Server offline while called."),
            Error::ServerAlreadyExists() => write!(f, "Server files already exist"),
            Error::ThreadError(ref a) => write!(f, "Error while creating {} thread for server", a),
            Error::ServerProcessExited() => {
                write!(f, "Server processes needed but has unexpectedly exited.")
            }
            Error::ServerAlreadyOnline() => write!(f, "Attempted to start server whilst online"),
            Error::ServerStillStarting() => {
                write!(f, "Attempted to stop server whilst mid-loading")
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
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
    /// let manager = serbo::Manager::new();
    /// manager.create();
    /// ```
    /// # Remarks
    /// Returns a result that contains the version of the server
    pub fn create(&mut self) -> Result<()> {
        if self.exists() {
            return Err(Error::ServerAlreadyExists());
        }

        match fs::create_dir("server") {
            Ok(()) => println!("Server directory created"),
            Err(e) => println!("Error creating server directory: {}", e),
        };

        self.update()?;

        Ok(())
    }

    /// Returns an Option<t> containing a [Instance](struct.Instance.html) that representing currently online server
    /// # Examples
    /// ```
    /// let manager = server::Manager::new();
    /// //Returns an Option
    /// let instance: server::Instance = manager.get().unwrap();
    /// ```
    /// # Remarks
    /// Queries the currently online server, for get to return, must have been launched by calling [start](struct.Manager.html#method.start)
    pub fn get(&mut self) -> Option<&mut Instance> {
        match &mut self.server {
            Some(a) => match a.is_valid() {
                Ok(b) => match b {
                    true => Some(a),
                    false => None,
                },
                Err(_) => None,
            },
            None => None,
        }
    }

    /// Checks if server files exist for a given id
    pub fn exists(&mut self) -> bool {
        Path::new(&format!("server/{}", self.jar_name)).exists()
    }

    /// Checks if the server is online
    /// # Remarks
    /// Queries the currently online servers, must have been launched by calling [start](struct.Manager.html#method.start)
    pub fn is_online(&mut self) -> bool {
        self.get().is_some()
    }

    /// Launches a server
    pub fn start(&mut self) -> Result<u32> {
        if !self.exists() {
            return Err(Error::ServerFilesMissing());
        }

        if self.server.is_some() {
            Err(Error::ServerAlreadyOnline())
        } else {
            let mut command = Command::new("java");
            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .args(&["-Xmx4G", "-Xms4G", "-jar", &self.jar_name, "nogui"])
                .current_dir("server");
            let mut serv_inst = Instance {
                server_process: command.spawn()?,
                stdout_join: None,
                starting: Arc::new(RwLock::new(true)),
            };

            let stdout = match serv_inst.server_process.stdout.take() {
                Some(e) => e,
                None => return Err(Error::ThreadError("stdout".to_string())),
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
            return Err(Error::ServerStillStarting());
        }
        Err(Error::ServerOffline())
    }

    /// OPs user with specified name
    pub fn op(&mut self, name: &str) -> Result<()> {
        if let Some(inst) = &mut self.server {
            match inst.send(format!("/op {}", name)) {
                Ok(()) => println!("Op'd {}", name),
                Err(e) => println!("Error Op'ing {}: {}", name, e),
            };
            return Ok(());
        }
        Err(Error::ServerOffline())
    }

    /// OPs user with specified name
    pub fn deop(&mut self, name: &str) -> Result<()> {
        if let Some(inst) = &mut self.server {
            match inst.send(format!("/deop {}", name)) {
                Ok(()) => println!("Deop'd {}", name),
                Err(e) => println!("Error Deop'ing {}: {}", name, e),
            };
            return Ok(());
        }
        Err(Error::ServerOffline())
    }

    /// Deletes a server's files
    /// # Remarks
    /// Stops the server if it's currently running
    pub fn delete(&mut self) -> Result<()> {
        self.stop()?;
        if !self.exists() {
            return Err(Error::ServerFilesMissing());
        }
        fs::remove_dir_all("server")?;
        Ok(())
    }

    /// Changes a server's version
    /// # Remarks
    /// Stops the server if it's currently running
    pub fn update(&mut self) -> Result<()> {
        // TODO: seperate to reusable functions
        // TODO: allow remove mod
        // TODO: allow github/jenkins mods
        // TODO: only update loader if newer version available
        let client = reqwest::Client::new();

        if self.is_online() {
            match self.stop() {
                Ok(()) => println!("Server stopped"),
                Err(e) => println!("Error stopping server: {}", e),
            }
        }

        let mut rt = tokio::runtime::Runtime::new()?;

        use serde_xml_rs::from_reader;

        let mut config_lock = config::CONFIG_LOCK.lock().unwrap();

        let fabric: Metadata = rt.block_on(async {
            from_reader(
                client
                    .get(&format!("{}maven-metadata.xml", FABRIC_INSTALLER))
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap()
        });

        let ver = fabric.versioning.latest.data;

        if !config_lock.installer_version.eq(&ver) {
            println!("Updating installer");

            let som = rt.block_on(async {
                client
                    .get(&format!(
                        "{0}{1}/fabric-installer-{1}.jar",
                        FABRIC_INSTALLER, ver
                    ))
                    .send()
                    .await
                    .unwrap()
                    .bytes()
                    .await
                    .unwrap()
            });

            match fs::File::create("./server/fabric-installer.jar") {
                Ok(mut file) => file.write_all(&som).unwrap(),
                Err(e) => println!("Error downloading installer: {}", e),
            };

            config_lock.update_installer_version(ver);
        }

        do_eula();

        let mut install = Command::new("java")
            .args(&[
                "-jar",
                "fabric-installer.jar",
                "server",
                "-downloadMinecraft",
            ])
            .current_dir("server")
            .stdout(Stdio::piped())
            .spawn()?;

        let a = BufReader::new(install.stdout.take().unwrap())
            .lines()
            .nth(3)
            .unwrap()
            .unwrap();
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

        for mc_mod in config::CONFIG.curseforge_mods.iter() {
            let _ = rt.block_on(async {
                println!("Checking mod: {}", mc_mod.mod_name);

                let info: ModInfo = serde_json::from_str(
                    &client
                        .get(&format!("{}{}", MOD_INFO, mc_mod.mod_id))
                        .send()
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap(),
                )
                .unwrap();

                for item in info.game_version_latest_files {
                    if item.game_version.eq(game_version) {
                        if config_lock.is_new(mc_mod) {
                            config_lock.new_mod(MCMod::copy(mc_mod));
                        } else {
                            if config_lock.is_same_version(mc_mod) {
                                break;
                            }
                            
                            config_lock.update_file_id(mc_mod.mod_id, item.project_file_id, mc_mod.platform);
                        }

                        print!("Downloading mod: {}, ", mc_mod.mod_name);

                        let mod_url = client
                            .get(&format!(
                                "{}{}/file/{}/download-url",
                                MOD_INFO, mc_mod.mod_id, item.project_file_id
                            ))
                            .send()
                            .await
                            .unwrap()
                            .text()
                            .await
                            .unwrap();
                        let mod_bytes = client
                            .get(&mod_url)
                            .send()
                            .await
                            .unwrap()
                            .bytes()
                            .await
                            .unwrap();

                        match fs::File::create(&format!("./server/mods/{}", item.project_file_name))
                        {
                            Ok(mut file) => file.write_all(&mod_bytes).unwrap(),
                            Err(e) => println!("Error saving mod: {}", e),
                        };

                        println!("Done");

                        break;
                    }
                }
            });
        }

        Ok(())
    }

    fn update_mods() {}
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
    pub fn is_valid(&mut self) -> Result<bool> {
        match self.server_process.try_wait()? {
            Some(_) => Ok(false),
            None => Ok(true),
        }
    }

    fn process_check(&mut self) -> Result<()> {
        match self.is_valid()? {
            true => Ok(()),
            false => Err(Error::ServerProcessExited()),
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
            None => return Err(Error::ThreadError("stdin".to_string())),
        };

        let mut writer = BufWriter::new(stdin);
        writeln!(writer, "{}", msg)?;
        writer.flush()?;

        Ok(())
    }
}

pub fn do_eula() {
    let _ = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("server/eula.txt")
        .and_then(|mut file| file.write(b"eula = true"));
}
