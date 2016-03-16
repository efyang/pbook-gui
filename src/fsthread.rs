use std::path::PathBuf;
use std::fs;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};

pub enum FsCommand {
    Copy(PathBuf, PathBuf), // source, dest
    Remove(PathBuf),
    Stop,
}

pub enum FsUpdate {
    Failed(String),
}

pub struct FsThread {
    command_recv: Receiver<FsCommand>,
    update_send: Sender<FsUpdate>,
}

impl FsThread {
    pub fn spawn() -> (Sender<FsCommand>, Receiver<FsUpdate>) {
        unimplemented!();
    }

    fn update(&mut self) -> Result<(), String> {
        unimplemented!();
    }

    fn handle_command(&mut self, command: FsCommand) {
        unimplemented!();
    }
}
