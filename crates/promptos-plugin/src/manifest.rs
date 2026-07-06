use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMeta,
    pub permissions: PluginPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissions {
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub filesystem: bool,
    #[serde(default)]
    pub api_keys: Vec<String>,
}

impl Default for PluginPermissions {
    fn default() -> Self {
        Self {
            network: false,
            filesystem: false,
            api_keys: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub permissions: PluginPermissions,
}

impl From<PluginManifest> for PluginInfo {
    fn from(m: PluginManifest) -> Self {
        Self {
            name: m.plugin.name,
            version: m.plugin.version,
            author: m.plugin.author,
            description: m.plugin.description,
            permissions: m.permissions,
        }
    }
}

impl PluginManifest {
    pub fn parse(toml_content: &str) -> Result<Self, String> {
        toml::from_str(toml_content).map_err(|e| format!("Invalid plugin manifest: {}", e))
    }

    pub fn default_manifest(name: &str) -> Self {
        Self {
            plugin: PluginMeta {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                author: "PromptOS User".to_string(),
                description: format!("{} plugin for PromptOS", name),
            },
            permissions: PluginPermissions::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parse() {
        let toml = r#"
[plugin]
name = "test-plugin"
version = "1.0.0"
author = "Test Author"
description = "A test plugin"

[permissions]
network = false
filesystem = false
api_keys = []
"#;
        let manifest = PluginManifest::parse(toml).unwrap();
        assert_eq!(manifest.plugin.name, "test-plugin");
        assert!(!manifest.permissions.network);
    }

    #[test]
    fn test_default_manifest() {
        let manifest = PluginManifest::default_manifest("my-plugin");
        assert_eq!(manifest.plugin.name, "my-plugin");
        assert_eq!(manifest.plugin.version, "1.0.0");
    }
}
