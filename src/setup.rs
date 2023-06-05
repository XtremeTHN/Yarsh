use directories::ProjectDirs;
use rustyline::config;
use std::fs::{create_dir_all, File};
use std::io::{Read,Write};
use chrono::Utc;
use serde::{Serialize, Deserialize};
use serde_yaml;
use log::{error};

use simplelog::{CombinedLogger, LevelFilter, TermLogger, WriteLogger, TerminalMode, ColorChoice, Config};

#[derive(Serialize, Deserialize, Clone)]
pub struct YamlConfiguration {
    pub logs_configurations: LogConfig
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LogConfig {
    pub write_to_file: bool,
    pub write_to_stdout: bool,
}

impl Default for YamlConfiguration {
    fn default() -> Self {
        YamlConfiguration { logs_configurations: LogConfig { write_to_file: true, write_to_stdout: false } }
    }
}

#[macro_export] macro_rules! set_config_key {
    ($struct:expr, $field:ident, $value:expr) => {
        {
            // Creamos una nueva instancia del struct modificando el campo deseado
            let mut modified_struct = $struct;
            modified_struct.$field = $value;
            modified_struct
        }
    };
}

pub fn setup() -> () {
    let base_dir = ProjectDirs::from("", "", "yarp");
    if base_dir.is_none() {
        println!("Couldn't initialize log file");
        eprintln!("Default data directory is null");
        ()
    };

    let mut log_dir = base_dir.clone().unwrap().data_dir().to_path_buf();
    let config_dir = base_dir.clone().unwrap().config_dir().to_path_buf();
    log_dir.push("logs");

    let mut preferences_file = config_dir.clone();
    preferences_file.push("preferences.yml");

    if let Err(err) = create_dir_all(log_dir.clone()) {
        error!("Couldn't initialize log file");
        error!("{}", err);
        ()
    }
    if let Err(err) = create_dir_all(config_dir.clone()) {
        error!("Couldn't initialize log file");
        error!("{}", err);
        ()
    }

    if !preferences_file.exists() {
        match File::create(preferences_file) {
            Ok(mut file) => {
                let general_conf: YamlConfiguration = Default::default();

                let conf_yaml = serde_yaml::to_string(&general_conf);
                if let Err(err) = file.write_all(conf_yaml.unwrap().as_bytes()) {
                    error!("Couldn't initialize log file");
                    error!("{}", err);
                    ()
                }
            }
            Err(err) => {
                error!("Couldn't initialize log file");
                error!("{}", err);
                ()
            }
        }
    }
    let time = Utc::now();
    let mut log_filename = log_dir.clone();
    log_filename.push(format!("yarp-{}.log", time.format("%Y-%m-%d_%H_%M_%S")));

    // Configurar el logger de archivo
    match File::create(log_filename) {

        Ok(log_file) => {
            let configs: YamlConfiguration = load_conf();

            let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = Vec::new();

            if configs.logs_configurations.write_to_file {
                loggers.push(WriteLogger::new(LevelFilter::Info, Config::default(), log_file));
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
        eprintln!("Couldn't read config file. Defaulting config values...");
        eprintln!("Base dir object is null");
        return default_values;
    } else {
        let mut config_dir = base_dir.clone().unwrap().config_dir().to_path_buf();
        config_dir.push("preferences.yml");

        match File::open(config_dir) {
            Ok(mut file) => {
                let mut buffer = String::new();
                let content = file.read_to_string(&mut buffer);
                if let Err(err) = content {
                    eprintln!("Couldn't read config file. Defaulting config values...");
                    eprintln!("Cannot read config file: {}", err);
                    return default_values;
                }
                let confs: YamlConfiguration = serde_yaml::from_str(&buffer).unwrap_or_else(|err| {
                    eprintln!("Couldn't read config file. Defaulting config values...");
                    eprintln!("{}", err);
                    return default_values;
                });
                return confs;
            }
            Err(err) => {
                eprintln!("Couldn't read config file. Defaulting config values...");
                eprintln!("Cannot open config file: {}", err);
                return default_values;
            }
        }
    }
}

pub fn open_config() -> Result<File, String> {
    let base_dir = ProjectDirs::from("", "", "yarp");
    if let Some(proj_obj) = base_dir {
        let mut config_path = proj_obj.config_dir().to_path_buf();
        config_path.push("preferences.yml");

        let conf_file_obj = File::create(config_path);
        match conf_file_obj {
            Ok(file) => {
                Ok(file)
            }
            Err(err) => {
                error!("setup::open_config(): Error while trying to open the config file");
                error!("setup::open_config(): {}", err);
                Err(String::from("Couldnt open the config file"))
            }
        }

    } else {
        error!("setup::open_config(): Couldnt get the data directory by an unkown error");
        Err(String::from("Couldnt open the config file"))
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
                            error!("commands::write_conf: Cannot write to the config file: {}", err);
                            error!("commands::write_conf Debug Info: Ocurred in setup.rs in line {}", line!());
                            error!("commands::write_conf: No changes has been applyed");
                            ()
                        }
                    },
                    Err(err) => {
                        error!("commands::write_conf: Cannot write to the config file: {}", err);
                        error!("commands::write_conf Debug Info: Ocurred in setup.rs in line {}", line!());
                        error!("commands::write_conf: No changes has been applyed");
                        ()
                    }
                }
            }
            Err(err) => {
                error!("commands::write_conf: Cannot write to the config file: {}", err);
                error!("commands::write_conf Debug Info: Ocurred in setup.rs in line {}", line!());
                error!("commands::write_conf: No changes has been applyed");
                ()
            }
        }
    }
}