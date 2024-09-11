use std::{
    collections::{HashMap, HashSet},
    error::Error,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{error::Result, Parser};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    configuration: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Configuration {
    /// If not selected, the first enabled plugin will be used as an alternative.
    default_plugin: Option<String>,
    plugins: Vec<PluginConfiguration>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PluginConfiguration {
    /// Plugin's name, must be unique
    name: String,
    /// Determine the plugin is enabled or not
    enabled: bool,
    /// Executable to run
    exec_path: String,
    /// Unix socket path to communicate with
    socket_path: String,
    /// Pass arguments to the plugin
    arguments: Option<Vec<String>>,
}

const FALLBACK_CONFIGURATION_FILE_PATH: &str = "/usr/share/runixeer/config.ron";

/// Resolves a symbolic link to its final destination path.
fn traverse_symlink(symlink_path: PathBuf) -> std::io::Result<PathBuf> {
    match symlink_path.read_link() {
        Ok(symlink_path) => {
            if symlink_path.is_symlink() {
                traverse_symlink(symlink_path)
            } else {
                Ok(symlink_path)
            }
        }
        Err(e) => {
            eprintln!("Error reading the file, {}", e);
            Err(e)
        }
    }
}

/// If the `--configuration` argument is provided but points to an invalid file, the code first
/// checks if XDG_CONFIG_HOME is set and uses it to construct a path to `./runixeer/config.ron`.
/// If that path doesn’t exist, it falls back to the user’s home directory
/// `~/.config/runixeer/config.ron`. If the file is not found there either, it defaults to
/// the predefined path `/usr/share/runixeer/config.ron`
fn get_configuration_path(args: Args) -> PathBuf {
    let default_configuration = PathBuf::from_str(FALLBACK_CONFIGURATION_FILE_PATH).unwrap();
    let _home_config_or_default = || -> PathBuf {
        match std::env::var("HOME") {
            Ok(config_home) => {
                let config_home_path = Path::new(&config_home);
                let config_path = Path::new(".config/runixeer/config.ron");
                let config_path = config_home_path.join(config_path);
                match config_path.exists() {
                    true if config_path.is_file() => config_path,
                    true if config_path.is_symlink() => match traverse_symlink(config_path) {
                        Ok(config_path) => {
                            if config_path.is_file() {
                                config_path
                            } else {
                                default_configuration.clone()
                            }
                        }
                        Err(_) => default_configuration.clone(),
                    },
                    _ => default_configuration.clone(),
                }
            }
            Err(_) => default_configuration.clone(),
        }
    };
    let _xdg_config_or_default = || -> PathBuf {
        match std::env::var("XDG_CONFIG_HOME") {
            Ok(xdg_config_home) => {
                let xdg_config_home_path = Path::new(&xdg_config_home);
                let config_path = Path::new("./runixeer/config.ron");
                let config_path = xdg_config_home_path.join(config_path);
                match config_path.exists() {
                    true if config_path.is_file() => config_path,
                    true if config_path.is_symlink() => match traverse_symlink(config_path) {
                        Ok(config_path) => {
                            if config_path.is_file() {
                                config_path
                            } else {
                                default_configuration.clone()
                            }
                        }
                        Err(_) => default_configuration.clone(),
                    },
                    _ => _home_config_or_default(),
                }
            }
            Err(_) => _home_config_or_default(),
        }
    };
    match args.configuration {
        Some(configuration_path) => {
            let configuration_path = Path::new(&configuration_path);
            match configuration_path.exists() {
                true if configuration_path.is_file() => configuration_path.to_owned(),
                true if configuration_path.is_symlink() => {
                    match traverse_symlink(configuration_path.to_path_buf()) {
                        Ok(config_path) => {
                            if config_path.is_file() {
                                config_path
                            } else {
                                _xdg_config_or_default()
                            }
                        }
                        Err(_) => _xdg_config_or_default(),
                    }
                }
                _ => _xdg_config_or_default(),
            }
        }
        None => _xdg_config_or_default(),
    }
}

struct PluginManager {
    enabled_plugins: HashSet<String>,
    started_plugins: Vec<String>,
    default_plugin: String,
    /// Plugin's Name -> PluginConfiguration
    plugins: HashMap<String, PluginConfiguration>,
}

#[derive(Debug)]
enum PluginManagerError {
    /// 1st argument is the name of the plugin.
    /// Plugin is not found in the configuration file.
    PluginNotFoundInConfiguration(String),
    /// 1st argument is the name of the plugin.
    /// Plugin is found in configuration file but exec_path is not pointing to
    /// an existing location.
    PluginNotFound(String),
    ///
    NoDefaultPlugin,
}

impl std::fmt::Display for PluginManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl std::error::Error for PluginManagerError {}

impl PluginManager {
    pub fn from_configuration(configuration: Configuration) -> Result<Self, Box<dyn Error>> {
        let mut enabled_plugins = HashSet::new();
        let mut map = HashMap::new();
        let mut default_plugin = None;
        if configuration.default_plugin.is_some() {
            default_plugin = configuration.default_plugin;
        }
        let mut is_first = true;
        for plugin in configuration.plugins {
            if plugin.enabled {
                if is_first {
                    default_plugin = Some(plugin.name.clone());
                    is_first = false;
                }
                enabled_plugins.insert(plugin.name.clone());
            }
            map.insert(plugin.name.clone(), plugin);
        }
        if default_plugin.is_some() {
            Ok(PluginManager {
                enabled_plugins,
                started_plugins: vec![],
                default_plugin: default_plugin.unwrap(),
                plugins: map,
            })
        } else {
            Err(Box::new(PluginManagerError::NoDefaultPlugin))
        }
    }
    pub fn start_enabled_plugins(&self) -> Result<(), Box<dyn Error>> {
        for plugin in self.enabled_plugins.clone() {
        }
        todo!()
    }
    pub fn start_default_plugin(&self) -> Result<(), Box<dyn Error>> {
        self.start_plugin(self.default_plugin.clone())
    }
    pub fn start_plugin(&self, name: String) -> Result<(), Box<dyn Error>> {
        match self.plugins.get(&name) {
            Some(plugin_config) => {
                if !Path::new(&plugin_config.exec_path).exists() {
                    return Err(Box::new(PluginManagerError::PluginNotFound(name)));
                }
                todo!()
            }
            None => {
                eprintln!("Plugin Not Found: {}", name);
                Err(Box::new(PluginManagerError::PluginNotFoundInConfiguration(
                    name,
                )))
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    let configuration_path = get_configuration_path(args);
    let configuration_file_contents =
        std::fs::read_to_string(configuration_path).expect("Could not read configuration file:");
    let configuration: Configuration = ron::from_str(&configuration_file_contents)
        .expect("Could not parse the configuration file:");
    let pluginmanager = PluginManager::from_configuration(configuration).expect("Could not instantiate Plugin Manager via configuration file.");
    pluginmanager.start_enabled_plugins().expect("Could not start the enabled plugins.");
    todo!()
}
