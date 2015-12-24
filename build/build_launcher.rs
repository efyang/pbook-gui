use std::path::Path;
use std::process::Command;

pub fn build(src_dir: &Path, dist_dir: &Path) {
    if cfg!(windows) {
        Command::new("g++")
            .arg(src_dir.join("launcher.cpp").to_str().unwrap())
            .arg("-o")
            .arg(dist_dir.join("pbook-launcher.exe").to_str().unwrap())
            .arg("-Os")
            .arg("-mwindows")
            .arg("-static")
            .arg("-static-libstdc++")
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute process {}", e));
    } else {
        Command::new("g++")
            .arg(src_dir.join("launcher.cpp").to_str().unwrap())
            .arg("-o")
            .arg(dist_dir.join("pbook-launcher").to_str().unwrap())
            .arg("-Os")
            .arg("-static")
            .arg("-static-libstdc++")
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute process {}", e));
    }
}
