use std::{thread, path::{PathBuf, Path}, env::{set_current_dir, current_dir}};
use rustyline::error::ReadlineError;
use crossterm::style::Stylize;
use rustyline::{DefaultEditor};
use directories::ProjectDirs;
use shellwords::split;
use log::{info};

mod commands;
mod setup;

use commands::{Builtin, run_external_command};


fn main() {
    setup::setup();
    setup::load_conf();
    info!("Log init successfull");
    /*let thread = thread::spawn(|| {
        load_python_plugin_init_files();
    });*/

    let mut rl = DefaultEditor::new().unwrap();
    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    
    let mut prompt = format!("{} >> ", current_dir().unwrap().to_string_lossy());
    loop {
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                if let Err(err) = rl.add_history_entry(line.as_str()) {
                    println!("{}: History cannot be saved", "Error".red());
                    println!("{}", err);
                }
                let shell_cmd = split(&line);
                if let Err(err) = shell_cmd {
                    println!("Debug: Cannot parse command: {}", err);
                    continue;
                }
                let shell_cmd = shell_cmd.unwrap();
                
                if let Some(unknown_cmd) = shell_cmd.get(0) {
                    match unknown_cmd.as_str() {
                        "ls" => {
                            if shell_cmd.get(1).is_some() {
                                if let Err(err) = Builtin::list_cmd(shell_cmd[1].clone()) {
                                    println!("ls: {}", err);
                                };
                            } else {
                                if let Err(err) = Builtin::list_cmd(".".to_string()) {
                                    println!("ls: {}", err);
                                };
                            }
                        },
                        "cd" => {
                            if shell_cmd.get(1).is_some() {
                                let binded = &shell_cmd[1].clone();
                                let root = Path::new(binded);
                                if let Err(err) = set_current_dir(root) {
                                    println!("cd: {}", err);
                                } else {
                                    let binded = current_dir().unwrap();
                                    prompt = format!("{} >> ", binded.to_string_lossy());
                                }
                            }
                        }

                        "echo" => {
                            println!("{}", shell_cmd.iter().skip(1).cloned().collect::<Vec<String>>().join(" "));
                        }

                        "clear" => {
                            if let Err(_) = Builtin::clear_screen() {
                                println!("clear: Error while trying to clear the terminal")
                            };
                        }

                        "read" => {
                            if let Some(file_path) = shell_cmd.get(1) {
                                if file_path == "-f" {
                                    if let Some(file_path2) = shell_cmd.get(2) {
                                        Builtin::read_file(PathBuf::from(file_path2), true);
                                    } else {
                                        println!("{}: you need to specify the file path to be readed", "read".green());
                                        continue;
                                    }
                                } else {
                                    Builtin::read_file(PathBuf::from(file_path), false);
                                }
                            } else {
                                println!("{}: you need to specify the file path to be readed", "read".green());
                                continue;
                            }
                        }

                        "config" => {
                            Builtin::config_cmd(shell_cmd);
                        }

                        "exit" => break,
                        &_ => {
                            let sh_cmd = shell_cmd[0].to_string();
                            let sd = shell_cmd.clone();
                            let obj = thread::spawn(move || {
                                run_external_command(&sh_cmd, Some(sd.clone()));
                            });
                            
                        }
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                continue;
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    #[cfg(feature = "with-file-history")]
    rl.save_history("history.txt");
    //thread.join();
}