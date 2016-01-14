use std::sync::mpsc::{Sender, Receiver};
use std::io::prelude::*;
use std::fs::File;
use std::fs;
use std::io::BufWriter;
use std::time::Duration;
use std::env::current_exe;
use hyper::client::Client;
use data::*;

pub const MILLI_TIMEMOUT: u64 = 500;
// NOTE
// make a Downloader struct which would allow for storage of download path and buffer size
// url, download path, buffer size

pub struct Downloader {
    url: String,
    download_path: PathBuf,
    cmd_recv: Receiver<TpoolCmdMsg>,
    progress_send: Sender<TpoolProgressMsg>,
}

impl Downloader {
    pub fn new(download: Download,
               cmd_recv: Receiver<TpoolCmdMsg>,
               progress_send: Sender<TpoolProgressMsg>)
               -> Downloader {
        Downloader {
            url: download.get_url().to_string(),
            download_path: download.get_path().to_owned(),
            cmd_recv: cmd_recv,
            progress_send: progress_send,
        }
    }

    pub fn update(&mut self) -> Result<(), String> {
        unimplemented!();
    }
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
    client.set_read_timeout(Some(Duration::from_millis(MILLI_TIMEMOUT)));
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
