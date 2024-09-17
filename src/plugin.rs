use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginConfiguration {
    /// Plugin's name, must be unique
    pub name: String,
    /// Determine the plugin is enabled or not
    pub enabled: bool,
    /// Executable to run
    pub exec_path: String,
    /// Unix socket path to communicate with
    pub socket_path: String,
    /// Pass arguments to the plugin
    pub arguments: Option<Vec<String>>,
}

pub struct Plugin {
    pub name: String,
    pub enabled: bool,
    /// Is plugin loaded by PM?
    pub loaded: bool,
    pub exec_path: String,
    pub socket_path: String,
    pub arguments: Option<Vec<String>>,
}

impl From<Plugin> for PluginConfiguration {
    fn from(plugin: Plugin) -> Self {
        Self {
            name: plugin.name,
            enabled: plugin.enabled,
            exec_path: plugin.exec_path,
            socket_path: plugin.socket_path,
            arguments: plugin.arguments,
        }
    }
}

/// `loaded` will be false by default
impl From<PluginConfiguration> for Plugin {
    fn from(plugin_config: PluginConfiguration) -> Self {
        Self {
            name: plugin_config.name,
            enabled: plugin_config.enabled,
            loaded: false,
            exec_path: plugin_config.exec_path,
            socket_path: plugin_config.socket_path,
            arguments: plugin_config.arguments,
        }
    }
}
