use std::sync::mpsc::{Sender, Receiver};
use std::io::prelude::*;
use std::io::{Error, BufWriter, ErrorKind};
use std::fs::{File, copy, create_dir_all};
use std::time::Duration;
use hyper::client::Client;
use hyper::client::response::Response;
use hyper::header::ContentLength;
use data::*;
use constants::CONNECT_MILLI_TIMEMOUT;
use helper::name_to_fname;

pub struct Downloader {
    name: String,
    url: String,
    id: u64,
    // download_path: PathBuf,
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
        let dlname = download.name().to_owned();
        Downloader {
            name: dlname.clone(),
            url: download.url().to_owned(),
            id: download.id(),
            cmd_recv: cmd_recv,
            progress_send: progress_send,
            filepath: download.path().to_owned().join(name_to_fname(&dlname)),
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
            if let Err(e) = create_dir_all(&self.filepath.parent().expect("No such dir parent")) {
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

        Ok(())
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
                            panic!("Error");
                        }
                    }
                }
            }
        }

        Ok(())
    }
    
    #[allow(dead_code)]
    fn send_message(&self, message: String) {
        self.progress_send
            .send((self.id, DownloadUpdate::Message(message)))
            .expect("Failed to send message");
    }
    
    #[allow(dead_code)]
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
                        let finish_msg = "File copied successfully".to_owned();
                        self.progress_send
                            .send((self.id, DownloadUpdate::Message(finish_msg)))
                            .expect("Failed to send copy finish");
                    }
                    Err(e) => {
                        let error_msg = make_chdir_error(e, "copy");
                        self.progress_send
                            .send((self.id, DownloadUpdate::Message(error_msg)))
                            .expect("Failed to send error");
                        return;

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

