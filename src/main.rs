use std::{thread, path::{PathBuf, Path}, env::{set_current_dir, current_dir}, process::Child, sync::{mpsc, atomic::AtomicBool, atomic::Ordering, Arc}};
use rustyline::error::ReadlineError;
use crossterm::style::Stylize;
use rustyline::{DefaultEditor};
use directories::ProjectDirs;
use shellwords::split;
use libc::{kill, pid_t, SIGTERM};
use log::{info, error};

mod commands;
mod setup;

use commands::{Builtin, wait_for_command, run_external_command};


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

    let mut current_command_pid: Arc<u32> = Arc::new(0);
    
    let should_stop = Arc::new(AtomicBool::new(false));
    let daemon_should_stop = Arc::clone(&should_stop);

    let (sv, rv) = mpsc::channel::<u32>();

    let ctrlc_sender = sv.clone();
    let ctrlc_ccppid = Arc::clone(&current_command_pid);
    ctrlc::set_handler(move || {
        ctrlc_sender.send(*ctrlc_ccppid);
    });

    let killer = thread::spawn(move || {
        while daemon_should_stop.load(Ordering::Relaxed) {
            match rv.recv() {
                Ok(pid) => {
                    let res = unsafe { kill(pid as pid_t, SIGTERM) };
                    if res == -1 {
                        error!("thread : killer : loop (while) : match : Ok(pid): Cannot kill process with pid of {pid}");
                        error!("thread : killer : loop (while) : match : Ok(pid): kill() command of the crate libc returned -1");
                        println!("yarp: Couldnt kill process with pid of {pid}");
                        continue;
                    }
                }
                Err(err) => {
                    error!("thread : killer : loop (while) : match : Err(err): Error while trying to recieve pid from the main thread");
                    error!("thread : killer : loop (while) : match : Err(err): {}", err);
                    println!("yarp: Error while trying to kill the process");
                    continue;
                }
            }
        }
    });

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
                            Builtin::list_cmd(shell_cmd);
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
                            // if let Some(file_path) = shell_cmd.get(1) {
                            //     if file_path == "-f" {
                            //         if let Some(file_path2) = shell_cmd.get(2) {
                            //             Builtin::read_file(PathBuf::from(file_path2), true);
                            //         } else {
                            //             println!("{}: you need to specify the file path to be readed", "read".green());
                            //             continue;
                            //         }
                            //     } else {
                            //         Builtin::read_file(PathBuf::from(file_path), false);
                            //     }
                            // } else {
                            //     println!("{}: you need to specify the file path to be readed", "read".green());
                            //     continue;
                            // }
                            Builtin::read_file(shell_cmd);
                        }

                        "config" => {
                            Builtin::config_cmd(shell_cmd);
                        }

                        "exit" => break,
                        &_ => {
                            let sh_cmd = shell_cmd[0].to_string();

                            if let Ok(output_obj) = run_external_command(&shell_cmd.join(" ")) {
                                let mut unwraped_output = output_obj.unwrap();
                                current_command_pid = Arc::new(unwraped_output.id());
                                unwraped_output.wait();
                            }
                            
                        }
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("yarp: If you want to exit the prompt, you need to execute the command 'exit'");
                continue;
            },
            Err(ReadlineError::Eof) => {
                println!("yarp: If you want to exit the prompt, you need to execute the command 'exit'");
                continue;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    #[cfg(feature = "with-file-history")]
    rl.save_history("history.txt");
    //thread.join();
}