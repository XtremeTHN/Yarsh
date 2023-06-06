use crossterm::style::Stylize;
use crossterm::{execute, terminal};
use crossterm::cursor::MoveTo;
use log::{info, debug, error};
use std::path::Path;
use std::process::Child;
use std::time::SystemTime;
use std::{env, fs, process::{Command, Stdio}, path::PathBuf};
use std::io::{self, Write, Read, BufRead};
use is_executable::IsExecutable;
use term_size::dimensions;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;

use crate::setup::{self, open_config, write_conf};
use walkdir::WalkDir;

pub fn columnize_text(items: &Vec<String>) {
    info!("commands::columnize_text(): Columnizing text...");
    if let Some((width, _)) = dimensions() {
        let longest_item_len = items.iter().map(|item| item.len()).max().unwrap_or(0);
        let num_columns = width / (longest_item_len + 2).max(1);
        let num_rows = (items.len() as f64 / num_columns as f64).ceil() as usize;

        for row in 0..num_rows {
            for column in 0..num_columns {
                let index = row + column * num_rows;
                if let Some(item) = items.get(index) {
                    print!("{:<width$}", item, width = longest_item_len + 2);
                }
            }
            println!();
        }
    } else {
        error!("commands::columnize_text(): Cannot retrieve terminal width. Columnizing without formating...");
        for item in items {
            println!("{}", item);
        }
    }
}

pub fn wait_for_command(pid: u32) {
    let pid = Pid::from_raw(pid as i32); // Reemplaza con el PID del proceso que deseas verificar
    loop {
        match waitpid(pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => {
                continue;
            }
            Ok(WaitStatus::Exited(_, code)) => {
                break;
            }
            Ok(WaitStatus::Signaled(_, _, _)) => {
                break;
            }
            Ok(WaitStatus::Stopped(_, _)) => {
                break;
            }
            Ok(WaitStatus::Continued(_)) => {
                break;
            }
            Ok(WaitStatus::PtraceEvent(_, _, _)) => {
                break;
            }
            Ok(WaitStatus::PtraceSyscall(_)) => {
                break;
            }
            Err(err) => {
                println!("No se pudo obtener el estado del proceso con PID {}.", pid);
                println!("{err}");
                break;
            }
        }
    }
}

fn format_system_time(time: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn find_executable_command(executable_name: &str) -> Option<PathBuf> {
    // Obtener la variable PATH
    let path_var = env::var("PATH").unwrap_or_else(|_| String::new());
    
    // Dividir la variable PATH en una lista de directorios
    let paths: Vec<_> = env::split_paths(&path_var).collect();
    
    // Buscar el ejecutable en cada directorio del PATH
    let mut found = None;
    for path in paths {
        let executable_path = path.join(executable_name);
        if fs::metadata(&executable_path).is_ok() {
            info!("commands::run_external_command(): Founded an executable on '{}'", executable_path.as_os_str().to_str().unwrap());
            found = Some(executable_path);
            break;
        }
    }
    found
}

// pub fn run_external_command(input: Vec<String>) -> Result<Child, &str>{
//         let executable_name = &input[0];
//         let found = find_executable_command(executable_name);
        
//         if let Some(cmd2) = executable_name.find("|") {
//             let executable_name_2 = executable_name.clone();
//             let (_, cmd) = executable_name_2.split_at(cmd2);
//             find_executable_command(cmd);
//         } else {
//             // Si el ejecutable no se encuentra
//             if let Some(executable_path) = found {
//                 if let Some(mut prog_args) = args {
//                     prog_args.drain(0..1);
//                     // Ejecutar el archivo
//                     info!("commands::run_external_command(): Executing program...");

//                     let output = Command::new(executable_path)
//                         .args(prog_args)
//                         .stdout(Stdio::inherit())
//                         .spawn();
                    
//                     Ok(output.unwrap())
//                 } else {
//                     let output = Command::new(executable_path)
//                         .stdout(Stdio::inherit())
//                         .spawn();
                    
//                     Ok(output.unwrap())
//                 }
//             } else {
//                 println!("yarp: unknown command: {}", executable_name);
//                 Err("Executable not found")
//             }
//         }
// }

pub fn run_external_command(command: &str) -> Result<Option<Child>, &str> {
    // Dividir el comando en partes separadas por el carácter '|'
    let commands: Vec<&str> = command.trim().split('|').collect();
    
    // Procesar cada comando en el pipeline
    let mut previous_output = None;
    for (index, cmd) in commands.iter().enumerate() {
        // Dividir el comando en partes separadas por espacios en blanco
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
        
        // Verificar si hay un ejecutable en la primera parte del comando
        if let Some(executable) = parts.first() {
            let found = find_executable_command(executable);
            
            // Si el ejecutable no se encuentra
            if let Some(executable_path) = found {
                // Configurar las opciones de redirección de entrada/salida
                let stdout = if index < commands.len() - 1 {
                    // Si no es el último comando, redirigir la salida al siguiente comando
                    Stdio::piped()
                } else {
                    // Si es el último comando, heredar la salida estándar del proceso padre
                    Stdio::inherit()
                };
                let stdin = previous_output.map_or(Stdio::inherit(), |output: Child| Stdio::from(output.stdout.unwrap()));
                
                // Ejecutar el comando
                let child_process = Command::new(executable_path)
                    .args(&parts[1..])
                    .stdout(stdout)
                    .stdin(stdin)
                    .spawn();
                
                // Verificar si se pudo ejecutar el comando
                match child_process {
                    Ok(child) => {
                        // Obtener la salida estándar del proceso actual para usarla como entrada en el siguiente comando
                        previous_output = Some(child);
                        if index == commands.len() - 1 {
                            info!("{:?}", previous_output);
                            return Ok(previous_output);
                        }
                    }
                    Err(_) => {
                        return Err("Failed to execute command");
                    }
                }
            } else {
                println!("yarp: unknown command: {}", executable);
                return Err("Executable not found");
            }
        }
    }
    Ok(None)
}

pub struct Builtin {}

impl Builtin {
    pub fn config_cmd(arguments: Vec<String>) {
        let mut configs = setup::load_conf();
        if let Some(operation) = arguments.get(1) {
            match operation.as_str() {
                "-l" => {
                    println!("{}: Listing values...", "config".blue());
                    println!("{} ({}):", "Logs Configurations".bold(), "logs_configurations".green());
                    println!("  write_to_file: {}", configs.logs_configurations.write_to_file);
                    println!("  write_to_stdout: {}", configs.logs_configurations.write_to_stdout);
                },
                "-s" => {
                    let section = arguments.get(2);
                    let key = arguments.get(3);
                    let value = arguments.get(4);

                    if key.is_none() || section.is_none() || value.is_none() {
                        println!("{}: Insufficient arguments, expected 2 found {}", "config".blue(), arguments.len() - 2);
                        ()
                    } else {
                        match section.clone().unwrap().as_str() {
                            "logs_configurations" => {
                                match key.clone().unwrap().as_str() {
                                    "write_to_file" => {
                                        configs.logs_configurations.write_to_file = value.clone().unwrap().parse().unwrap();
                                        write_conf(configs);
                                    }
                                    "write_to_stdout" => {
                                        configs.logs_configurations.write_to_stdout = value.clone().unwrap().parse().unwrap();
                                        write_conf(configs);
                                    }
                                    &_ => {
                                        println!("{}: No such field", "config".blue());
                                        ()
                                    }
                                }
                            }
                            &_ => {
                                println!("{}: No such section", "config".blue());
                            }
                        }
                    }

                },
                "-g" => {
                    let section = arguments.get(2);
                    let key = arguments.get(3);

                    if key.is_none() || section.is_none() {
                        println!("{}: Insufficient arguments, expected 2 found {}", "config".blue(), arguments.len() - 2);
                        ()
                    } else {
                        info!("{}", section.clone().unwrap().as_str());
                        match section.clone().unwrap().as_str() {
                            "logs_configurations" => {
                                match key.clone().unwrap().as_str() {
                                    "write_to_file" => {
                                        println!("{}: {}", "Value".cyan(), configs.logs_configurations.write_to_file); 
                                    }
                                    "write_to_stdout" => {
                                        println!("{}: {}", "Value".cyan(), configs.logs_configurations.write_to_stdout); 
                                    }
                                    &_ => {
                                        println!("{}: No such field", "config".blue());
                                        ()
                                    }
                                }
                            }
                            &_ => {
                                println!("{}: No such section", "config".blue());
                            }
                        }
                    }
                }
                &_ => {}
            }
        }
    }
    pub fn clear_screen() -> io::Result<()> {
        info!("commands::clear_screan(): Trying to clear terminal");
        let mut stdout = io::stdout();
        execute!(stdout, terminal::Clear(terminal::ClearType::All), MoveTo(0, 0))?;
        stdout.flush()?;
        info!("Succes");
        Ok(())
    }

    pub fn read_file(file_path: PathBuf, force_read: bool) {
        let mut is_exec = file_path.is_executable();
        if force_read {
            is_exec = false;
        }
        match is_exec {
            false => {
                let file = fs::File::open(file_path.clone());
                info!("commands::Builtin::read_file(): Reading file...");
                match file {
                    Ok(mut file_obj) => {
                        info!("commands::Builtin::read_file(): Creating buffer...");
                        let mut buffer = String::new();
                        match file_obj.read_to_string(&mut buffer) {
                            Ok(_) => println!("{}", buffer),
                            Err(err) => {
                                error!("commands::Builtin::read_file(): Error while trying to save file content to the buffer");
                                error!("commands::Builtin::read_file(): {}", err);
                                println!("{}: Error while trying to read {}", "read".green(), file_path.as_path().to_str().unwrap());
                                println!("{}: More information in the logs (You can use the 'logs last_log' command)", "read".green());
                            }
                        }
                    }
                    Err(err) => {
                        error!("commands::Builtin::read_file(): Error while trying to open the file");
                        error!("commands::Builtin::read_file(): {}", err);
                        println!("{}: Error while trying to read {}", "read".green(), err);
                        println!("{}: More information in the logs (You can use the 'logs last_log' command)", "read".green());
                    }
                }
            }
            true => {
                info!("commands::Builtin::read_file(): Executable file detected");
                info!("commands::Builtin::read_file(): Reading metadata...");
                let metadata = fs::metadata(file_path.clone());
                match metadata {
                    Ok(md_obj) => {
                        println!("{}: Executable file detected! Reading metadata instead of the file content...", "read".green());
                        let mut metadata_formated: Vec<String> = vec![];

                        let lm = md_obj.accessed().unwrap_or(SystemTime::now());
                        
                        let mut read_write_perms = String::new();
                        if md_obj.permissions().readonly() {
                            read_write_perms.push_str(format!("{}: Read only", "Permissions".blue().bold()).as_str());
                        } else {
                            read_write_perms.push_str(format!("{}: Read and write", "Permissions".blue().bold()).as_str());
                        }
                        
                        metadata_formated.push(
                            format!("{}: {}", "Last Modified".blue().bold(), format_system_time(lm)),
                        );
                        metadata_formated.push(
                            read_write_perms
                        );
                        info!("commands::Builtin::read_file(): Showing metadata...");
                        columnize_text(&metadata_formated);
                    }
                    Err(err) => {
                        error!("commands::Builtin::read_file(): Error while trying to get file metadata");
                        error!("commands::Builtin::read_file(): {}", err);
                        println!("read: Couldnt read {} metadata", file_path.as_path().to_str().unwrap());
                        ()
                    }
                }
            }
        }
    }
    
    pub fn list_cmd(work_dir: String) -> Result<(), String> {
        info!("commands::list_cmd(): Listing files in {}", work_dir);
        let work_dir_convertion = PathBuf::from(&work_dir);
        let mut colored_vector: Vec<String> = vec![];
        //if let Ok(iterator) = work_dir_convertion.read_dir() {
        match work_dir_convertion.read_dir() {
            Ok(iterator) => {
                for x in iterator {
                    if x.as_ref().unwrap().path().is_dir() {
                        let file = x.unwrap().path().clone();
                        colored_vector.push(file.file_name().unwrap().to_str().unwrap().blue().to_string());
                        
                    } else if x.as_ref().unwrap().path().is_symlink() {
                        let file = x.unwrap().path().clone();
                        colored_vector.push(file.file_name().unwrap().to_str().unwrap().green().to_string());
                    } else {
                        let file = x.unwrap().path().clone();
                        colored_vector.push(file.file_name().unwrap().to_str().unwrap().yellow().to_string());
                    }
                }
                columnize_text(&colored_vector);
                Ok(())
            }
            Err(err) => Err(format!("Cannot read the directory. Error: {}", err)),
        }
    }
}