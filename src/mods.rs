use super::config::Platform;

#[derive(Debug, Serialize, Deserialize)]
pub struct MCMod {
    pub mod_name: String,
    pub mod_id: u64,
    pub file_id: Option<u64>,
    pub platform: Platform,
}

impl MCMod {
    pub fn new(mod_name: String, mod_id: u64, platform: Platform, file_id: Option<u64>) -> Self {
        Self {
            mod_name,
            mod_id,
            file_id,
            platform,
        }
    }

    pub fn copy(mc_mod: &Self) -> Self {
        Self {
            mod_name: mc_mod.mod_name.clone(),
            mod_id: mc_mod.mod_id,
            file_id: mc_mod.file_id,
            platform: mc_mod.platform,
        }
    }
}
