use std::{
    collections::HashMap, error::Error, io::Write, os::unix::net::UnixStream, thread::JoinHandle,
};

use crate::{config::Configuration, plugin::Plugin};

pub struct PluginManager {
    default_plugin: String,
    /// Plugin's Name -> PluginConfiguration
    pub plugins: HashMap<String, Plugin>,
    /// Threads are responsible to run the executables/plugins.
    pub threads: HashMap<String, JoinHandle<Result<std::process::ExitStatus, std::io::Error>>>,
}

#[derive(Debug)]
pub enum PluginManagerError {
    /// 1st argument is the name of the plugin.
    /// Plugin is not found in the configuration file.
    PluginNotFoundInConfiguration(String),
    /// 1st argument is the name of the plugin.
    /// Plugin is not found in configuration file.
    PluginNotFound(String),
    /// No plugin enabled but should be.
    NotEvenOneEnabledPlugin,
    /// Plugin's executable not found.
    ExecNotFound(String),
    /// Socket error
    SocketError(std::io::Error),
}

impl std::fmt::Display for PluginManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PluginManagerError {}

impl PluginManager {
    pub fn from_configuration(configuration: Configuration) -> Result<Self, Box<dyn Error>> {
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
            }
            map.insert(plugin.name.clone(), Plugin::from(plugin));
        }
        if default_plugin.is_some() {
            Ok(PluginManager {
                default_plugin: default_plugin.unwrap(),
                threads: HashMap::new(),
                plugins: map,
            })
        } else {
            // Above code makes sure if default_plugin is not specified in configuration file,
            // then the first enabled plugin is selected. But if there is no plugin enabled, who is
            // the default one?
            Err(Box::new(PluginManagerError::NotEvenOneEnabledPlugin))
        }
    }

    pub fn get_default_plugin(&self) -> &Plugin {
        self.plugins.get(&self.default_plugin).unwrap()
    }

    fn load_plugin(
        exec_path: String,
    ) -> JoinHandle<Result<std::process::ExitStatus, std::io::Error>> {
        std::thread::spawn(|| std::process::Command::new(exec_path).status())
    }

    pub fn start_plugin(&mut self, name: String) -> Result<(), Box<dyn Error>> {
        match self.plugins.get(&name) {
            Some(plugin) => {
                if !std::path::Path::new(&plugin.exec_path.clone()).exists() {
                    return Err(Box::new(PluginManagerError::ExecNotFound(
                        plugin.name.clone(),
                    )));
                }
                PluginManager::load_plugin(plugin.exec_path.clone());
                let plugin = Plugin {
                    loaded: true,
                    arguments: plugin.arguments.clone(),
                    socket_path: plugin.socket_path.clone(),
                    exec_path: plugin.exec_path.clone(),
                    name: plugin.name.clone(),
                    ..*plugin
                };
                match self.plugins.insert(name.clone(), plugin) {
                    Some(_previous) => Ok(()),
                    None => Err(Box::new(PluginManagerError::PluginNotFound(name))),
                }
            }
            None => Err(Box::new(PluginManagerError::PluginNotFound(name))),
        }
    }

    pub fn start_enabled_plugins(&mut self) -> Result<(), Vec<Box<dyn Error>>> {
        let mut errors = vec![];
        let mut que: Vec<String> = vec![];
        for (name, plugin) in &self.plugins {
            if plugin.enabled {
                if !std::path::Path::new(&plugin.exec_path.clone()).exists() {
                    errors.push(
                        Box::new(PluginManagerError::ExecNotFound(plugin.name.clone())).into(),
                    );
                }
                que.push(name.clone());
            }
        }
        for name in que {
            let plugin = self.plugins.get(&name).unwrap();
            let plugin = Plugin {
                loaded: true,
                arguments: plugin.arguments.clone(),
                socket_path: plugin.socket_path.clone(),
                exec_path: plugin.exec_path.clone(),
                name: plugin.name.clone(),
                ..*plugin
            };
            PluginManager::load_plugin(plugin.exec_path.clone());
            match self.plugins.insert(name.clone(), plugin) {
                Some(_previous) => {}
                None => {
                    errors.push(Box::new(PluginManagerError::PluginNotFound(name.clone())).into())
                }
            }
        }
        match errors.len() > 0 {
            true => Err(errors),
            _ => Ok(()),
        }
    }

    fn send_exit_action(socket_path: String) -> Result<(), Box<dyn Error>> {
        match UnixStream::connect(&socket_path) {
            Ok(mut unix_stream) => {
                let message = ron::to_string(&librunixeer::Action::Exit).unwrap();
                match unix_stream.write(message.as_bytes()) {
                    Ok(_) => match unix_stream.shutdown(std::net::Shutdown::Write) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(Box::new(e)),
                    },
                    Err(e) => Err(Box::new(e)),
                }
            }
            Err(e) => {
                eprintln!("Could not connect to the socket at {}", socket_path);
                Err(Box::new(e))
            }
        }
    }

    pub fn exit_one(&mut self, name: String) -> Result<(), Box<dyn Error>> {
        match self.plugins.get(&name) {
            Some(plugin) => PluginManager::send_exit_action(plugin.socket_path.clone()),
            None => todo!(),
        }
    }

    pub fn exit_all(&self) -> Result<(), Vec<Box<dyn Error>>> {
        let mut errors = vec![];
        for (_name, plugin) in &self.plugins {
            if plugin.loaded {
                if let Err(e) = PluginManager::send_exit_action(plugin.socket_path.clone()) {
                    errors.push(e);
                }
            }
        }
        match errors.len() > 0 {
            true => Err(errors),
            _ => Ok(()),
        }
    }
}
