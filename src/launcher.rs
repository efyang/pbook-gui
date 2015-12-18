use std::env::current_exe;
use std::path::Path;
use std::process::Command;

#[cfg(windows)]
const EXE_NAME: &'static str = "pbook-gui.exe";

#[cfg(not(windows))]
const EXE_NAME: &'static str = "pbook-gui";

fn main() {
    let current_path = current_exe().unwrap();
    let current_dir = current_path.parent()
                                  .unwrap()
                                  .clone();
    let exe_paths = [Path::new("bin").join(EXE_NAME), Path::new(EXE_NAME).to_path_buf()];
    let mut found = false;
    for entry in exe_paths.iter() {
        let fullpath = current_dir.join(entry);
        if fullpath.exists() {
            match Command::new(fullpath.clone()).spawn() {
                Ok(_) => {}
                Err(_) => println!("Could not run \"{}\"", fullpath.display()),
            }
            found = true;
            break;
        }
    }
    if !found {
        println!("Error: No valid \"{}\" executable found.", EXE_NAME);
    }
}
