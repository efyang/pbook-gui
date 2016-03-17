use std::path::PathBuf;
use std::fs;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use helper::Ignore;

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
        let (update_send, update_recv) = channel();
        let (command_send, command_recv) = channel();
        let builder = thread::Builder::new()
                          .stack_size(100000)
                          .name("File System Thread".to_owned());
        builder.spawn(move || {
            let mut run = true;
            let fsthread = FsThread::new(command_recv, update_send);
            while run {
                if let Ok(command) = fsthread.command_recv.try_recv() {
                    match command {
                        FsCommand::Copy(source, dest) => {
                            if let Err(e) = fs::copy(source, dest) {
                                fsthread.update_send.send(FsUpdate::Error(e.to_string())).ignore();
                            }
                        }
                        FsCommand::Remove(path) => {
                            if path.is_dir() {
                                // remove directory
                                if fs::read_dir(&path).unwrap().count() > 0 {
                                    // files in directory
                                    fsthread.update_send
                                            .send(FsUpdate::Error(format!("Files in directory: \
                                                                           {:?}",
                                                                          path)))
                                            .ignore();
                                } else {
                                    if let Err(e) = fs::remove_dir(path) {
                                        fsthread.update_send
                                                .send(FsUpdate::Error(e.to_string()))
                                                .ignore();
                                    }
                                }
                            } else {
                                // remove file
                                if let Err(e) = fs::remove_file(path) {
                                    fsthread.update_send
                                            .send(FsUpdate::Error(e.to_string()))
                                            .ignore();
                                }
                            }
                        }
                        FsCommand::Stop => {
                            run = false;
                        }
                    }
                }
            }
        }).expect("Failed to spawn FsThread");
        return (command_send, update_recv);
    }
}
