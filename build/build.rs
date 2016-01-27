use std::process::Command;

pub fn main() {
    // update git submodule
    Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
}
