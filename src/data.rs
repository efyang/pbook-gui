use hyper::client::*;
use hyper::header::ContentLength;
use std::time::Duration;
use std::hash::{Hash, Hasher, SipHasher};
pub use std::path::{Path, PathBuf};

pub enum DownloadUpdate {
    Message(String),
    Amount(usize),
}

pub type TpoolProgressMsg = (u64, DownloadUpdate);
pub type GuiCmdMsg = (String, Option<u64>);
pub type TpoolCmdMsg = GuiCmdMsg;

pub trait ToDownloads {
    fn to_downloads(&self) -> Vec<Download>;
}

impl ToDownloads for Vec<Category> {
    fn to_downloads(&self) -> Vec<Download> {
        let mut downloads = Vec::new();
        for category in self.iter() {
            downloads.extend(category.downloads().iter().cloned());
        }
        downloads
    }
}

#[derive(Debug, Clone)]
pub struct Category {
    name: String,
    downloads: Vec<Download>,
}

impl Category {
    pub fn new(name: String, downloads: Vec<Download>) -> Category {
        Category {
            name: name,
            downloads: downloads,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn add_download(&mut self, download: Download) {
        self.downloads.push(download);
    }

    pub fn set_enable_state_all(&mut self, enable_state: bool) {
        for dl in self.downloads.iter_mut() {
            dl.enabled = enable_state;
        }
    }

    pub fn begin_download(&mut self, download_id: &u64) -> Result<(), String> {
        let mut exists = false;
        for dl in self.downloads.iter_mut() {
            if &dl.id == download_id {
                dl.start_download();
                exists = true;
                break;
            }
        }
        if exists {
            Ok(())
        } else {
            Err(format!("No such download id {} exists.", download_id))
        }
    }

    pub fn increment_download_progress(&mut self, download_id: &u64, increment: usize) -> Result<(), String> {
        for dl in self.downloads.iter_mut() {
            if &dl.id == download_id {
                return dl.increment_progress(increment);
            }
        }
        // default if not found
        Err(format!("No such download id {} exists.", download_id))
    }

    pub fn downloads(&self) -> &[Download] {
        &self.downloads
    }
}

pub fn get_dl_id(name: &str, url: &str) -> u64 {
    let mut hasher = SipHasher::new();
    format!("{}{}", name, url).hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone)]
pub struct Download {
    name: String,
    url: String,
    enabled: bool,
    dlinfo: Option<DownloadInfo>, /* optional depending on whether
                                   * its currently being downloaded */
    pub id: u64,
}

impl Download {
    pub fn new(name: &str, url: &str) -> Download {
        // id is siphash of name + url
        Download {
            name: name.to_string(),
            url: url.to_string(),
            enabled: false,
            dlinfo: None,
            id: get_dl_id(name, url),
        }
    }

    pub fn increment_progress(&mut self, increment: usize) -> Result<(), String> {
        if let Some(ref mut dlinfo) = self.dlinfo {
            dlinfo.increment_progress(increment);
            Ok(())
        } else {
            Err("Progress cannot be incremented because it is not downloading.".to_string())
        }
    }

    pub fn is_downloading(&self) -> bool {
        self.dlinfo.is_some()
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn start_download(&mut self) {
        self.dlinfo = Some(DownloadInfo::new());
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_path(&self) -> PathBuf {
        self.clone().dlinfo.unwrap().get_path()
    }
}

#[derive(Debug, Clone)]
pub struct DownloadInfo {
    failed: bool,
    progress: usize,
    total: usize,
    elapsed: Duration,
    path: PathBuf,
}

impl DownloadInfo {
    pub fn new() -> DownloadInfo {
        DownloadInfo {
            failed: false,
            progress: 0,
            total: 0,
            elapsed: Duration::new(0, 0),
            path: PathBuf::new(),
        }
    }

    pub fn with_total(total: usize) -> DownloadInfo {
        DownloadInfo {
            failed: false,
            progress: 0,
            total: total,
            elapsed: Duration::new(0, 0),
            path: PathBuf::new(),
        }
    }

    pub fn get_progress(&self) -> usize {
        self.progress
    }

    pub fn increment_progress(&mut self, increment: usize) {
        self.progress += increment;
    }

    fn get_path(&self) -> PathBuf {
        self.path.to_path_buf()
    }
}
