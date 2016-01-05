use std::io::prelude::*;
use std::fs::File;
use std::fs;
use std::io::BufWriter;
use std::time::Duration;
use std::path::Path;
use std::env::current_exe;
use hyper::client::Client;

pub const MILLI_TIMEMOUT: usize = 500;

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
    let mut buf: [u8; 1] = [0; 1];
    //let mut buf: [u8; 16] = [0; 16];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(_) => {
                outfile.write(&buf).unwrap();
                //outfile.write(&buf[..n]).unwrap();
            }
            Err(e) => panic!(e),
        }
    }
    outfile.flush().unwrap()
}

pub fn get_url_filename(url: &str) -> Option<&str> {
    url.split('/').last()
}
