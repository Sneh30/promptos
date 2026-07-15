use crate::manifest::{PluginInfo, PluginManifest};
use log::{info, warn};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

pub struct PluginHost {
    plugins: Mutex<HashMap<String, PluginInstance>>,
    allowed_dirs: Vec<String>,
}

struct PluginInstance {
    info: PluginInfo,
    wasm_bytes: Vec<u8>,
    enabled: bool,
}

impl PluginHost {
    pub fn new() -> Self {
        Self {
            plugins: Mutex::new(HashMap::new()),
            allowed_dirs: vec![],
        }
    }

    pub fn load_plugin(&self, path: &str) -> Result<String, String> {
        info!("Plugin load — path={}", path);
        if !Path::new(path).exists() {
            warn!("Plugin load — file not found: {}", path);
            return Err(format!("Plugin file not found: {}", path));
        }

        let wasm_data = std::fs::read(path)
            .map_err(|e| format!("Cannot read plugin file: {}", e))?;
        let wasm_len = wasm_data.len();

        let manifest_path = Path::new(path).with_extension("toml");
        let manifest = if manifest_path.exists() {
            let content = std::fs::read_to_string(&manifest_path)
                .map_err(|e| format!("Cannot read manifest: {}", e))?;
            PluginManifest::parse(&content)?
        } else {
            let name = Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            PluginManifest::default_manifest(name)
        };

        let info: PluginInfo = manifest.into();
        let name = info.name.clone();

        let mut plugins = self.plugins.lock().map_err(|e| e.to_string())?;
        plugins.insert(name.clone(), PluginInstance {
            info,
            wasm_bytes: wasm_data,
            enabled: true,
        });

        info!("Plugin loaded — name={}, bytes={}", name, wasm_len);
        Ok(name)
    }

    pub fn unload_plugin(&self, name: &str) -> Result<(), String> {
        info!("Plugin unload — name={}", name);
        let mut plugins = self.plugins.lock().map_err(|e| e.to_string())?;
        plugins.remove(name);
        Ok(())
    }

    pub fn enable_plugin(&self, name: &str) -> Result<(), String> {
        info!("Plugin enable — name={}", name);
        let mut plugins = self.plugins.lock().map_err(|e| e.to_string())?;
        if let Some(plugin) = plugins.get_mut(name) {
            plugin.enabled = true;
            Ok(())
        } else {
            warn!("Plugin enable — not found: {}", name);
            Err(format!("Plugin not found: {}", name))
        }
    }

    pub fn disable_plugin(&self, name: &str) -> Result<(), String> {
        info!("Plugin disable — name={}", name);
        let mut plugins = self.plugins.lock().map_err(|e| e.to_string())?;
        if let Some(plugin) = plugins.get_mut(name) {
            plugin.enabled = false;
            Ok(())
        } else {
            warn!("Plugin disable — not found: {}", name);
            Err(format!("Plugin not found: {}", name))
        }
    }

    pub fn get_info(&self, name: &str) -> Result<PluginInfo, String> {
        let plugins = self.plugins.lock().map_err(|e| e.to_string())?;
        plugins.get(name).map(|p| p.info.clone()).ok_or_else(|| format!("Plugin not found: {}", name))
    }

    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins
            .lock()
            .map(|p| p.values().map(|v| v.info.clone()).collect())
            .unwrap_or_default()
    }

    pub fn call_hook(&self, _name: &str, _hook: &str, _payload: &[u8]) -> Result<Vec<u8>, String> {
        // WASM execution would go here with wasmtime
        // For now, return empty result as placeholder
        info!("Plugin hook call — name={}, hook={} (WASM execution not yet implemented)", _name, _hook);
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_host_empty() {
        let host = PluginHost::new();
        assert!(host.list_plugins().is_empty());
    }

    #[test]
    fn test_plugin_host_load_nonexistent() {
        let host = PluginHost::new();
        let result = host.load_plugin("/nonexistent/plugin.wasm");
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_enable_disable() {
        // Can't easily test without a real WASM file
        // But we can test the error paths
        let host = PluginHost::new();
        assert!(host.enable_plugin("nonexistent").is_err());
        assert!(host.disable_plugin("nonexistent").is_err());
        assert!(host.get_info("nonexistent").is_err());
    }
}
