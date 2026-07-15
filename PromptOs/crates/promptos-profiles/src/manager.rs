use crate::profile::ModelProfile;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct ProfileManager {
    profiles: Mutex<HashMap<String, ModelProfile>>,
    profiles_dir: PathBuf,
}

impl ProfileManager {
    pub fn new(profiles_dir: &str) -> Self {
        Self {
            profiles: Mutex::new(HashMap::new()),
            profiles_dir: PathBuf::from(profiles_dir),
        }
    }

    pub fn load_defaults(&self) -> Result<usize, String> {
        let profiles = vec![
            crate::profile::anthropic_claude_3_5_sonnet(),
            crate::profile::openai_gpt4o(),
            crate::profile::google_gemini_1_5_pro(),
        ];

        let count = profiles.len();
        let mut map = self.profiles.lock().map_err(|e| e.to_string())?;
        for profile in profiles {
            map.insert(profile.profile.model_id.clone(), profile);
        }

        Ok(count)
    }

    pub fn load_from_dir(&self, dir: &str) -> Result<usize, String> {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        let mut map = self.profiles.lock().map_err(|e| e.to_string())?;

        for entry in
            std::fs::read_dir(dir_path).map_err(|e| format!("Cannot read profiles dir: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Cannot read entry: {}", e))?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "toml") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Cannot read profile {}: {}", path.display(), e))?;
                match toml::from_str::<ModelProfile>(&content) {
                    Ok(profile) => {
                        map.insert(profile.profile.model_id.clone(), profile);
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Invalid profile {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(count)
    }

    pub fn get(&self, model_id: &str) -> Option<ModelProfile> {
        self.profiles.lock().ok()?.get(model_id).cloned()
    }

    pub fn list(&self) -> Vec<ModelProfile> {
        self.profiles
            .lock()
            .map(|map| map.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn list_model_ids(&self) -> Vec<String> {
        self.profiles
            .lock()
            .map(|map| map.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn refresh_from_remote(&self, _url: &str) -> Result<usize, String> {
        tracing::info!("Remote profile refresh not yet implemented");
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_load_defaults() {
        let manager = ProfileManager::new("/tmp/profiles");
        let count = manager.load_defaults().unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_manager_get_profile() {
        let manager = ProfileManager::new("/tmp/profiles");
        manager.load_defaults().unwrap();
        let profile = manager.get("claude-3.5-sonnet");
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().profile.provider, "anthropic");
    }

    #[test]
    fn test_manager_get_nonexistent() {
        let manager = ProfileManager::new("/tmp/profiles");
        manager.load_defaults().unwrap();
        let profile = manager.get("nonexistent-model");
        assert!(profile.is_none());
    }

    #[test]
    fn test_manager_list() {
        let manager = ProfileManager::new("/tmp/profiles");
        manager.load_defaults().unwrap();
        let profiles = manager.list();
        assert_eq!(profiles.len(), 3);
    }
}
