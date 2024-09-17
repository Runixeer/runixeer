use crate::{args::Args, plugin::PluginConfiguration};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration {
    /// If not selected, the first enabled plugin will be used as an alternative.
    pub default_plugin: Option<String>,
    pub plugins: Vec<PluginConfiguration>,
}

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
pub fn get_configuration_path(args: Args) -> PathBuf {
    let default_configuration =
        PathBuf::from_str(crate::pinned::FALLBACK_CONFIGURATION_FILE_PATH).unwrap();
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
