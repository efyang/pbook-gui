use std::path::PathBuf;
use std::fs;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::io;

pub enum FsCommand {
    Copy(PathBuf, PathBuf), // source, dest
    Remove(PathBuf),
    Stop,
}

pub enum FsUpdate {
    Error(String),
}

pub struct FsThread {
    pub command_recv: Receiver<FsCommand>,
    pub update_send: Sender<FsUpdate>,
}

impl FsThread {
    fn new(command_recv: Receiver<FsCommand>, update_send: Sender<FsUpdate>) -> FsThread {
        FsThread {
            command_recv: command_recv,
            update_send: update_send,
        }
    }

    pub fn spawn() -> (Sender<FsCommand>, Receiver<FsUpdate>) {
        unimplemented!();
        let (update_send, update_recv) = channel();
        let (command_send, command_recv) = channel();
        let builder = thread::Builder::new();
        builder.stack_size(100000);
        builder.name("File System Thread".to_owned());
        builder.spawn(move || {
            let run = true;
            let fsthread = FsThread::new(command_recv, update_send);
            while run {
                match fsthread.command_recv.recv() {
                    
                }
            }
        });
        return (command_send, update_recv);
    }

    fn handle_command(&self, command: FsCommand) {
        unimplemented!();
    }
}
