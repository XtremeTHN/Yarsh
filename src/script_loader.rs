use rlua::{Lua,UserData, UserDataMethods, Error};
use crate::{setup::{self, YamlConfiguration}, commands::ExternalCommands};
use log::{error, info};
use std::{fs::File, io::{Read}, path::{PathBuf}, process::Child, sync::Arc};

#[derive(Clone)]
pub struct Alias {
    config: YamlConfiguration,
}

impl Alias {
    pub fn new() -> Self {
        Alias { config: setup::load_conf() }
    }
    pub fn set_alias(&mut self, cmd: &str, alias: &str) {
        self.config.terminal_config.alias.insert(String::from(cmd), String::from(alias));
        setup::write_conf(self.config.clone());
    }
}

impl UserData for Alias {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // Agrega los métodos de la clase a la tabla de métodos
        methods.add_method("new", |_, _, _: ()| {
            Ok(Alias::new())
        });
        methods.add_method("set_alias", |_, this, (cmd, alias): (String, String)| {
            let mut self_clone = this.clone();
            Ok(self_clone.set_alias(&cmd, &alias))
        });
    }
}


pub fn read_file(file: PathBuf) -> Result<String, String> {
    if !file.exists() {
        return Err(String::from("File doesn't exists"));
    }
    if file.is_file() {
        match File::open(file.clone()) {
            Ok(mut file_obj) => {
                let mut buffer = String::new();
                
                match file_obj.read_to_string(&mut buffer) {
                    Ok(_) => {
                        Ok(buffer)
                    },
                    Err(err) => {
                        error!("script_loader::read_file(): Error while trying to read {}", file.file_name().unwrap().to_str().unwrap());
                        error!("script_loader::read_file(): {err}");
                        Err(err.to_string())
                    }
                }
            },
            Err(err) => {
                error!("script_loader::read_file(): Error while trying to read {}", file.file_name().unwrap().to_str().unwrap());
                error!("script_loader::read_file(): {err}");
                Err(err.to_string())
            }
        }
    } else {
        Err(String::from("Not a file"))
    }
}

pub fn load(files: Vec<PathBuf>, external_cmds_obj: ExternalCommands) -> () {
    let lua_obj = Lua::new();
    for file in files {
        lua_obj.context(|ctx| {
            let globals = ctx.globals();

            let external = external_cmds_obj.clone();
            let execute_function = ctx.create_function(move |_, exec_name: String| {
                let child_result_obj = external.run_external_command(&exec_name);
                match child_result_obj {
                    Ok(obj) => {
                        if let Some(mut child) = obj {
                            child.wait();
                            Ok(())
                        } else {
                            Err(Error::RuntimeError(String::from("The run_external_command() returned None")))
                        }
                    }
                    Err(err) => {
                        Err(Error::RuntimeError(err.to_string()))
                    }
                }
            });

            match execute_function {
                Ok(func) => {
                    if let Err(err) = globals.set("exec", func) {
                        error!("script_loader::load(): Error while trying to set the exec() command to the lua context");
                        error!("script_loader::load(): {err}");
                    }
                }
                Err(err) => {
                    error!("script_loader::load(): Error while trying to create the exec() command");
                    error!("script_loader::load(): {err}");
                }
            }

            let alias_result = ctx.create_userdata(Alias::new());
            match alias_result {
                Ok(alias) => {
                    if let Err(err) = globals.set("Alias", alias) {
                        error!("script_loader::load(): Error while trying to set the Alias class to the lua context");
                        error!("script_loader::load(): {err}");
                        ()
                    }

                    match read_file(file) {
                        Ok(source) => {
                            if let Err(err) = ctx.load(&source).exec() {
                                error!("script_loader::load(): Exception ocurred in file: {}", "placeholder");
                                error!("{}", err);
                                println!("Failed to run scripts");
                            }
                        }
                        Err(_) => {
                            println!("yarp: Failed to read script file");
                            ()
                        }
                    }
                }
                Err(err) => {
                    println!("yarsh: Cannot load lua scripts because this error: {err}");
                }
            }
        });
    }
}