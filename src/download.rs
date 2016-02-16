use std::sync::mpsc::{Sender, Receiver};
use std::io::prelude::*;
use std::io::{Error, BufWriter, ErrorKind};
use std::fs::{File, copy};
use std::fs;
use std::time::Duration;
use std::env::current_exe;
use hyper::client::Client;
use hyper::client::response::Response;
use data::*;
use constants::CONNECT_MILLI_TIMEMOUT;

pub struct Downloader {
    name: String,
    url: String,
    id: u64,
    //download_path: PathBuf,
    cmd_recv: Receiver<TpoolCmdMsg>,
    progress_send: Sender<TpoolProgressMsg>,
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
        let dlname = download.get_name().to_string();
        Downloader {
            name: dlname.clone(),
            url: download.get_url().to_string(),
            id: download.get_id(),
            cmd_recv: cmd_recv,
            progress_send: progress_send,
            filepath: download.get_path().to_owned().join(name_to_fname(&dlname)),
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
        if let None = self.stream {
            match self.client.get(&self.url).send() {
                Ok(s) => {
                    self.stream = Some(s);
                }
                Err(e) => {
                    println!("geterror");
                    return Err(format!("{}", e));
                }
            }
        }

        if let None = self.outfile {
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

        Ok(())
    }

    pub fn update(&mut self) -> Result<(), String> {
        // check messages
        match self.cmd_recv.try_recv() {
            Ok(cmd) => {
                match &cmd.0 as &str {
                    "stop" => {
                        drop(self);
                        // kill thread
                        return Err("Thread stopped".to_string());
                    }
                    _ => {}
                }
            }
            Err(_) => {}
        }
        // download more bytes
        let mut finished = false;
        if let Some(ref mut outfile) = self.outfile {
            if let Some(ref mut stream) = self.stream {
                match stream.read(&mut self.buffer) {
                    Ok(0) => {
                        // Finished downloading
                        outfile.flush().expect("Failed to flush to outfile");
                        self.progress_send
                            .send((self.id, DownloadUpdate::Message("finished".to_owned())))
                            .expect("Failed to send message");
                        finished = true;
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
                            panic!("Error");
                        }
                    }
                }
            }
        }

        if finished {
            drop(self);
            panic!();
        }

        Ok(())
    }

    fn send_message(&self, message: String) {
        self.progress_send
            .send((self.id, DownloadUpdate::Message(message)))
            .expect("Failed to send message");
    }

    fn change_path(&mut self, newpath: &Path) {
        if newpath != self.filepath {
            // preexisting outfile
            if let Some(ref mut outfile) = self.outfile {
                // flush outfile
                match outfile.flush() {
                    Ok(_) => {}
                    Err(e) => {
                        let error_msg = make_chdir_error(e, "flush");
                        self.progress_send
                            .send((self.id, DownloadUpdate::Message(error_msg)))
                            .expect("Failed to send error");
                        return;
                    }
                }
                // copy over the file
                match copy(self.filepath.clone(), newpath) {
                    Ok(_) => {
                        self.filepath = newpath.to_path_buf();
                        let finish_msg = "File copied successfully".to_string();
                        self.progress_send
                            .send((self.id, DownloadUpdate::Message(finish_msg)))
                            .expect("Failed to send copy finish");
                    }
                    Err(e) => {
                        let error_msg = make_chdir_error(e, "copy");
                    }
                }
            } else {
                // make the file
                match File::create(newpath) {
                    Ok(f) => {
                        self.outfile = Some(BufWriter::new(f));
                    }
                    Err(e) => {
                        let error_msg = make_chdir_error(e, "fileopen");
                        self.progress_send
                            .send((self.id, DownloadUpdate::Message(error_msg)))
                            .expect("Failed to send error");
                        return;
                    }
                }
            }
        }
    }
}

fn make_chdir_error(errorstring: Error, kind: &str) -> String {
    format!("Failed to change directory: {} error: {}",
            kind,
            errorstring)
}

fn name_to_fname(s: &str) -> String {
    spaces_to_underscores(s) + ".pdf"
}

fn spaces_to_underscores(s: &str) -> String {
    s.replace(" ", "_")
}
