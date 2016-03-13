use std::sync::mpsc::{Sender, Receiver};
use std::io::prelude::*;
use std::io::{Error, BufWriter, ErrorKind};
use std::fs::{File, copy, create_dir_all, rename};
use std::time::Duration;
use hyper;
use hyper::client::Client;
use hyper::client::response::Response;
use hyper::header::ContentLength;
use data::*;
use constants::CONNECT_MILLI_TIMEMOUT;
use helper::name_to_fname;
use std::fs::metadata;

pub struct Downloader {
    name: String,
    url: String,
    id: u64,
    // download_path: PathBuf,
    cmd_recv: Receiver<TpoolCmdMsg>,
    progress_send: Sender<TpoolProgressMsg>,
    actualpath: PathBuf,
    filepath: PathBuf,
    client: Client,
    stream: Option<Response>,
    outfile: Option<BufWriter<File>>,
    buffer: [u8; 16],
}

impl Downloader {
    pub fn new(download: Download,
               cmd_recv: Receiver<TpoolCmdMsg>,
               progress_send: Sender<TpoolProgressMsg>)
               -> Downloader {
        let dlname = download.name().to_owned();
        let path = download.path().to_owned().join(name_to_fname(&dlname));
        Downloader {
            name: dlname.clone(),
            url: download.url().to_owned(),
            id: download.id(),
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
            buffer: [0; 16],
        }
    }

    pub fn begin(&mut self) -> Result<(), String> {
        if self.actualpath.exists() {
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
            return Err("Over try limit".to_owned());
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
                                println!("{:?}", ioerr.kind());
                                println!("geterror");
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
                        rename(&self.filepath, &self.actualpath)
                            .expect(&format!("Failed to rename tmp file: {:?} to {:?}",
                                             self.filepath,
                                             self.actualpath));
                        self.progress_send
                            .send((self.id, DownloadUpdate::Finished))
                            .expect("Failed to send message");
                        return Err("finished".to_owned());
                    }
                    Ok(n) => {
                        // got n bytes
                        outfile.write(&self.buffer[..n]).expect("IO write error");
                        self.progress_send
                            .send((self.id, DownloadUpdate::Amount(n)))
                            .expect("Failed to send message");
                    }
                    Err(e) => {
                        // Some error
                        if e.kind() != ErrorKind::WouldBlock {
                            println!("Error Type: {:?}", e.kind());
                            println!("Name: {}", self.name);
                            println!("Url: {}", self.url);
                            println!("Error: {:?}", e.into_inner());
                            return Err("Downloader Error".to_owned());
                        }
                    }
                }
            }
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
        let newpath = newdir.join(current_filename);
        self.change_path(&newpath);
    }

    fn change_path(&mut self, newpath: &Path) {
        if newpath != self.filepath {
            let mut message = None;
            let mut open_outfile = false;
            // preexisting outfile
            if let Some(ref mut outfile) = self.outfile {
                // flush outfile
                let mut flush_successful = true;
                match outfile.flush() {
                    Ok(_) => {}
                    Err(e) => {
                        flush_successful = false;
                        let error_msg = make_chdir_error(e, "flush");
                        message = Some(error_msg);
                    }
                }
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
            } else {
                // make the file
                match File::create(newpath) {
                    Ok(f) => {
                        self.outfile = Some(BufWriter::new(f));
                    }
                    Err(e) => {
                        let error_msg = make_chdir_error(e, "fileopen");
                        message = Some(error_msg);
                    }
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
}

fn make_chdir_error(errorstring: Error, kind: &str) -> String {
    format!("Failed to change directory: {} error: {}",
            kind,
            errorstring)
}
