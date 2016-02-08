use std::sync::mpsc::{Sender, Receiver};
use std::io::prelude::*;
use std::io::{Error, BufWriter};
use std::fs::{File, copy};
use std::fs;
use std::time::Duration;
use std::env::current_exe;
use hyper::client::Client;
use hyper::client::response::Response;
use data::*;
use constants::CONNECT_MILLI_TIMEMOUT;

// NOTE
// make a Downloader struct which would allow for storage of download path and buffer size
// url, download path, buffer size

pub struct Downloader {
    url: String,
    id: u64,
    download_path: PathBuf,
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
               progress_send: Sender<TpoolProgressMsg>,
               path: &Path)
               -> Downloader {
        Downloader {
            url: download.get_url().to_string(),
            id: download.get_id(),
            download_path: download.get_path().to_owned(),
            cmd_recv: cmd_recv,
            progress_send: progress_send,
            filepath: path.to_path_buf(),
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
                    return Err(format!("{}", e));
                }
            }
        }

        if let None = self.outfile {
            match File::open(&self.filepath) {
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
                        panic!(e);
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
        if newpath != self.download_path {
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

pub fn download_url_default(url: &str) {
    download_url(url, get_url_filename(url).unwrap());
}

pub fn download_url(url: &str, fileout: &str) {
    let ce = current_exe().unwrap();
    let cd = ce.parent().unwrap();
    let dldir = cd.join("downloads");
    fs::create_dir_all(dldir.clone()).unwrap();
    let filename = dldir.join(fileout);
    let mut client = Client::new();
    client.set_read_timeout(Some(Duration::from_millis(CONNECT_MILLI_TIMEMOUT)));
    let mut outfile = BufWriter::new(File::create(filename).unwrap());
    let mut stream = client.get(url).send().unwrap();
    let mut buf: [u8; 16] = [0; 16];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                outfile.write(&buf[..n]).unwrap();
            }
            Err(e) => panic!(e),
        }
    }
    outfile.flush().unwrap()
}

pub fn get_url_filename(url: &str) -> Option<&str> {
    url.split('/').last()
}
