use std::sync::mpsc::{Sender, Receiver};
use std::io::prelude::*;
use std::io::{Error, BufWriter, ErrorKind};
use std::fs::{File, copy, create_dir_all, rename, metadata, remove_file};
use std::time::Duration;
use hyper;
use hyper::client::Client;
use hyper::client::response::Response;
use hyper::header::ContentLength;
use data::*;
use constants::CONNECT_MILLI_TIMEMOUT;
use helper::{name_to_fname, name_to_dname, Ignore};
use std::thread::sleep;

pub struct Downloader {
    url: String,
    id: u64,
    category_name: Option<String>,
    cmd_recv: Receiver<TpoolCmdMsg>,
    progress_send: Sender<TpoolProgressMsg>,
    actualpath: PathBuf,
    filepath: PathBuf,
    client: Client,
    stream: Option<Response>,
    outfile: Option<BufWriter<File>>,
    buffer: [u8; 128],
}

impl Downloader {
    pub fn new(download: Download,
               cmd_recv: Receiver<TpoolCmdMsg>,
               progress_send: Sender<TpoolProgressMsg>)
        -> Downloader {
            let dlname = download.name().to_owned();
            let path = download.path().to_owned().join(name_to_fname(&dlname));
            Downloader {
                url: download.url().to_owned(),
                id: download.id(),
                category_name: download.category_name().to_owned(),
                cmd_recv: cmd_recv,
                progress_send: progress_send,
                actualpath: path.clone(),
                filepath: path.parent()
                    .unwrap()
                    .join(path.file_name()
                          .unwrap()
                          .to_str()
                          .unwrap()
                          .to_owned() + ".tmp"),
                          client: {
                              let mut client = Client::new();
                              client.set_read_timeout(Some(Duration::from_millis(CONNECT_MILLI_TIMEMOUT)));
                              client
                          },
                          stream: None,
                          outfile: None,
                          buffer: [0; 128],
            }
        }

    pub fn begin(&mut self) -> Result<(), String> {
        let actual_exists;
        let filepath_exists;
        {
            actual_exists = File::open(&self.actualpath).is_ok();
            filepath_exists = File::open(&self.actualpath).is_ok();
        }
        if actual_exists {
            // get file metadata (size)
            let filelength;
            if let Ok(metadata) = metadata(self.actualpath.clone()) {
                filelength = metadata.len();
            } else {
                filelength = 1;
            }
            self.progress_send
                .send((self.id, DownloadUpdate::SetSize(filelength as usize)))
                .expect("Failed to send content length");

            // send finished signal
            self.progress_send
                .send((self.id, DownloadUpdate::Finished))
                .expect("Failed to send finished");
            return Err("finished".to_owned());
        } else {
            if filepath_exists {
                // remove the preexisting tmp file
                remove_file(self.filepath.as_path()).expect(&format!("Failed to remove pre-existing .tmp file: {:?}", self.filepath));
            }
            if let Err(e) = self.get_url(0, 5) {
                return Err(e);
            }

            if let Some(ref stream) = self.stream {
                match stream.headers.get::<ContentLength>() {
                    Some(content_length) => {
                        self.progress_send
                            .send((self.id, DownloadUpdate::SetSize(**content_length as usize)))
                            .expect("Failed to send content length");
                    }
                    None => {}
                }
            }

            // NOTE: Need to make dir before making file

            if let None = self.outfile {
                if let Err(e) = create_dir_all(&self.filepath
                                               .parent()
                                               .expect("No such dir parent")) {
                    println!("dir creation error");
                    return Err(format!("{}", e));
                }

                match File::create(&self.filepath) {
                    Ok(f) => {
                        self.outfile = Some(BufWriter::new(f));
                    }
                    Err(e) => {
                        println!("fopen error");
                        return Err(format!("{}", e));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_url(&mut self, tries: usize, maxtries: usize) -> Result<(), String> {
        if tries >= maxtries {
            return Err(format!("Over connection try limit of {}.", maxtries));
        } else {
            if let None = self.stream {
                match self.client.get(&self.url).send() {
                    Ok(s) => {
                        self.stream = Some(s);
                        return Ok(());
                    }
                    Err(e) => {
                        if let hyper::Error::Io(ioerr) = e {
                            if ioerr.kind() == ErrorKind::WouldBlock {
                                return self.get_url(tries, maxtries);
                            } else {
                                return self.get_url(tries + 1, maxtries);
                            }
                        } else {
                            return Err(format!("{:?}", e));
                        }
                    }
                }
            } else {
                return Ok(());
            }
        }

    }

    pub fn update(&mut self) -> Result<(), String> {
        // check messages
        if let Ok(cmd) = self.cmd_recv.try_recv() {
            match cmd {
                TpoolCmdMsg::Remove(id) => {
                    if self.id == id {
                        return Err("stopped".to_owned());
                    }
                }
                TpoolCmdMsg::ChangeDir(newdir) => {
                    self.change_path_dir(&newdir);
                }
                TpoolCmdMsg::Stop => {
                    return Err("stopped".to_owned());
                }
            }
        }
        // download more bytes
        if let Some(ref mut outfile) = self.outfile {
            if let Some(ref mut stream) = self.stream {
                match stream.read(&mut self.buffer) {
                    Ok(0) => {
                        // Finished downloading
                        outfile.flush().expect("Failed to flush to outfile");
                        drop(outfile);
                        rename(&self.filepath, &self.actualpath).ignore();
                            //.expect(&format!("Failed to rename tmp file: {:?} to {:?}",
                                             //self.filepath,
                                             //self.actualpath));
                        self.progress_send
                            .send((self.id, DownloadUpdate::Finished))
                            //.expect("Failed to send message");
                            .ignore();
                        return Err("finished".to_owned());
                    }
                    Ok(n) => {
                        // got n bytes
                        outfile.write(&self.buffer[..n]).expect("IO write error");
                        self.progress_send
                            .send((self.id, DownloadUpdate::Amount(n)))
                            //.expect("Failed to send message");
                            .ignore();
                    }
                    Err(e) => {
                        // Some error
                        if e.kind() != ErrorKind::WouldBlock {
                            let kind = e.kind();
                            let error = e.into_inner().unwrap();
                            if "StringError(\"early eof\")" == &format!("{}", error) {
                                return Err(format!("Error: \n{:?} - {:?}", kind, "connection dropped - try redownloading the file."));
                            } else {
                                return Err(format!("Error: \n{:?} - {:?}", kind, error));
                            }
                        }
                    }
                }
            }
            sleep(Duration::new(0, 100));
        } else {
            sleep(Duration::new(0, 100000));
        }

        Ok(())
    }

    fn send_message(&self, message: String) {
        self.progress_send
            .send((self.id, DownloadUpdate::Message(message)))
            .expect("Failed to send message");
    }

    fn change_path_dir(&mut self, newdir: &Path) {
        let current_filename = self.filepath.file_name().unwrap().to_owned();
        let newpath;
        if let Some(ref category) = self.category_name {
            let category_dir = newdir.join(name_to_dname(category));
            create_dir_all(&category_dir).expect("Failed to create dir");
            newpath = category_dir.join(current_filename);
        } else {
            newpath = newdir.join(current_filename);
        }
        self.change_path(&newpath);
    }

    fn change_path(&mut self, newpath: &Path) {
        if newpath != self.filepath {
            // REFACTOR THIS LATER
            let mut message = None;
            let mut open_outfile = false;
            // preexisting outfile
            let mut is_some = false;
            let mut flush_successful = true;
            if let Some(ref mut outfile) = self.outfile {
                // flush outfile
                is_some = true;
                match outfile.flush() {
                    Ok(_) => {}
                    Err(e) => {
                        flush_successful = false;
                        let error_msg = make_chdir_error(e, "flush");
                        message = Some(error_msg);
                    }
                }

            } else {
                // make the file
                match File::create(newpath) {
                    Ok(f) => {
                        self.outfile = Some(BufWriter::new(f));
                        self.filepath = newpath.to_path_buf();
                    }
                    Err(e) => {
                        let error_msg = make_chdir_error(e, "fileopen");
                        message = Some(error_msg);
                    }
                }
            }

            if is_some {
                self.outfile = None;
                if flush_successful {
                    // copy over the file
                    match copy(self.filepath.clone(), newpath) {
                        Ok(_) => {
                            self.filepath = newpath.to_path_buf();
                            let finish_msg = "File copied successfully".to_owned();
                            message = Some(finish_msg);
                        }
                        Err(e) => {
                            let error_msg = make_chdir_error(e, "copy");
                            message = Some(error_msg);
                        }
                    }

                    open_outfile = true;
                }
            }
            // set outfile
            if open_outfile {
                match File::open(newpath) {
                    Ok(f) => {
                        self.outfile = Some(BufWriter::new(f));
                    }
                    Err(e) => {
                        let error_msg = make_chdir_error(e, "fileopen");
                        message = Some(error_msg);
                    }
                }
            }
            // send any message
            if let Some(msg) = message {
                self.send_message(msg);
            }
        }
    }

    pub fn send_panicked(&self, e: String) {
        self.progress_send.send((self.id, DownloadUpdate::Panicked(e))).ignore();
    }
}

fn make_chdir_error(errorstring: Error, kind: &str) -> String {
    format!("Failed to change directory: {} error: {}",
            kind,
            errorstring)
}
