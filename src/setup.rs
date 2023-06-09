use chrono::Utc;
use directories::ProjectDirs;
use log::error;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use std::io::{Read, Write};

use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct YamlConfiguration {
    pub logs_configurations: LogConfig,
    pub terminal_config: TermConfig,
    pub scripts_config: ScriptsConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TermConfig {
    pub prompt: String,
    pub alias: HashMap<String, String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScriptsConfig {
    pub files: Vec<PathBuf>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogConfig {
    pub write_to_file: bool,
    pub write_to_stdout: bool,
}

impl Default for YamlConfiguration {
    fn default() -> Self {
        YamlConfiguration {
            logs_configurations: LogConfig {
                write_to_file: true,
                write_to_stdout: false,
            },
            terminal_config: TermConfig { 
                prompt: String::from("{}"), 
                alias: HashMap::new(),
            },
            scripts_config: ScriptsConfig { 
                files: vec![],
            }
        }
    }
}

pub fn setup() -> () {
    let base_dir = ProjectDirs::from("", "", "yarp");
    if base_dir.is_none() {
        println!("Couldn't initialize log file");
        error!("Default data directory is null");
        ()
    };

    let mut log_dir = base_dir.clone().unwrap().data_dir().to_path_buf();
    let config_dir = base_dir.unwrap().config_dir().to_path_buf();
    log_dir.push("logs");

    let mut preferences_file = config_dir.clone();
    preferences_file.push("preferences.yml");

    if let Err(err) = create_dir_all(log_dir.clone()) {
        error!("Couldn't initialize log file");
        error!("{}", err);
        ()
    }
    if let Err(err) = create_dir_all(config_dir) {
        error!("Couldn't initialize log file");
        error!("{}", err);
        ()
    }

    if !preferences_file.exists() {
        // match File::create(preferences_file) {
        //     Ok(mut file) => {
        //         let general_conf: YamlConfiguration = Default::default();

        //         let conf_yaml = serde_yaml::to_string(&general_conf);
        //         if let Err(err) = file.write_all(conf_yaml.unwrap().as_bytes()) {
        //             error!("Couldn't initialize log file");
        //             error!("{}", err);
        //             ()
        //         }
        //     }
        //     Err(err) => {
        //         error!("Couldn't initialize log file");
        //         error!("{}", err);
        //         ()
        //     }
        // }
        write_conf(Default::default());
    }
    let time = Utc::now();
    let mut log_filename = log_dir.clone();
    log_filename.push(format!("yarp-{}.log", time.format("%Y-%m-%d_%H_%M_%S")));

    // Configurar el logger de archivo
    match File::create(log_filename) {
        Ok(log_file) => {
            let configs: YamlConfiguration = load_conf();

            let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = Vec::new();

            if configs.logs_configurations.write_to_stdout {
                loggers.push(TermLogger::new(
                    LevelFilter::Info,
                    Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ));
            }

            if configs.logs_configurations.write_to_file {
                loggers.push(WriteLogger::new(
                    LevelFilter::Info,
                    Config::default(),
                    log_file,
                ));
            }

            // Combinar ambos loggers
            if let Err(err) = CombinedLogger::init(loggers) {
                error!("Couldn't initialize log file");
                error!("{}", err);
                ()
            };
        }
        Err(err) => {
            error!("Couldn't initialize log file");
            error!("{}", err);
            ()
        }
    }
}

pub fn load_conf() -> YamlConfiguration {
    let base_dir = ProjectDirs::from("", "", "yarp");
    let default_values = Default::default();
    if base_dir.is_none() {
        error!("Couldn't read config file. Defaulting config values...");
        error!("Base dir object is null");
        return default_values;
    } else {
        let mut config_dir = base_dir.unwrap().config_dir().to_path_buf();
        config_dir.push("preferences.yml");

        match File::open(config_dir) {
            Ok(mut file) => {
                let mut buffer = String::new();
                let content = file.read_to_string(&mut buffer);
                if let Err(err) = content {
                    error!("Couldn't read config file. Defaulting config values...");
                    error!("Cannot read config file: {}", err);
                    return default_values;
                }
                let confs: YamlConfiguration =
                    serde_yaml::from_str(&buffer).unwrap_or_else(|err| {
                        error!("Couldn't read config file. Defaulting config values...");
                        error!("{}", err);
                        return default_values;
                    });
                return confs;
            }
            Err(err) => {
                error!("Couldn't read config file. Defaulting config values...");
                error!("Cannot open config file: {}", err);
                return default_values;
            }
        }
    }
}

pub fn write_conf(configs: YamlConfiguration) {
    let base_dir = ProjectDirs::from("", "", "yarp");
    if let Some(config_path) = base_dir {
        let mut conf = config_path.config_dir().to_path_buf();
        conf.push("preferences.yml");

        match File::create(conf) {
            Ok(mut config_file) => {
                let configs_to_str = serde_yaml::to_string(&configs);
                match configs_to_str {
                    Ok(conf_buff) => {
                        if let Err(err) = config_file.write_all(conf_buff.as_bytes()) {
                            error!(
                                "setup::write_conf: Cannot write to the config file: {}",
                                err
                            );
                            error!(
                                "setup::write_conf Debug Info: Ocurred in setup.rs in line {}",
                                line!()
                            );
                            error!("setup::write_conf: No changes has been applyed");
                            ()
                        }
                    }
                    Err(err) => {
                        error!(
                            "setup::write_conf: Cannot write to the config file: {}",
                            err
                        );
                        error!(
                            "setup::write_conf Debug Info: Ocurred in setup.rs in line {}",
                            line!()
                        );
                        error!("setup::write_conf: No changes has been applyed");
                        ()
                    }
                }
            }
            Err(err) => {
                error!(
                    "setup::write_conf: Cannot write to the config file: {}",
                    err
                );
                error!(
                    "setup::write_conf Debug Info: Ocurred in setup.rs in line {}",
                    line!()
                );
                error!("setup::write_conf: No changes has been applyed");
                ()
            }
        }
    }
}