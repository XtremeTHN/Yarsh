# Yet Another Rusty Shell (Yarsh)

This is a upgraded and with another objective of my project RustyShell.
Tecnically this project is a fork of RustyShell, so part of the code will be the same. But theres much more features now!

## Built-in commands

###### ls
You can list the files in the current folder:
```
/ >> ls
.git           src            target         settings.sh   README.md      
Cargo.toml     Cargo.lock     recent_log.py  .gitignore 
/ >>
```
###### cd
You can change the current directory by writing this command
```
/ >> cd foo/bar
/foo/bar >> 
```

###### echo
You can print messages into stdout by writting this
```
/ >> echo foo bar
foo bar
/ >>
```

###### clear
You can clear the terminal with the clear command
```
/ >> clear
```

###### read
Now you can read files with this new command! 
```
/ >> read foo.txt
Hello World!
/ >>
```
If you try to read a executable file it will show to you the file metadata
```
/ >> read foo
read: Executable file detected! Reading metadata instead of the file content...
Last Modified: 2023-06-05 20:04:02  Permissions: Read and write
/ >>
```
If you wanna force the read of the executable file, ,you can pass the -f argument
Note: If the file is a binary it will generate a error
```
/ >> read -f foo
echo Hello World
/ >>
```

###### config
Now theres a config file, by the moment, you only can edit where you want to write the logs.

With the `-l` argument you can list the current configurations
```
/ >> config -l
config: Listing values...
Logs Configurations (logs_configurations):
  write_to_file: true
  write_to_stdout: false
```

With the `-g` argument you can get an specific value of an argument
```
/ >> config -g logs_configurations write_to_file
Value: true
```

Lastly, with the `-s` argument you can change the value of specific field
```
/ >> config -s logs_configurations write_to_file true
```

## Extra features

###### More logging messages
More log messages for deugging purposes!. The logs will be stored at `/home/$USER/.local/share/yash/logs`

###### Pipelines
The pipelines are currently in development, but it works, you can make a pipe line with this syntax
`command_1 arguments | command_2 arguments`