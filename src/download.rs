use hyper::client::*;
use hyper::header::ContentLength;
use std::time::Duration;
use std::hash::{Hash, Hasher, SipHasher};

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

    pub fn begin_download(&mut self, download_id: &u64) {
        for dl in self.downloads.iter_mut() {
            if &dl.id == download_id {
                dl.start_download();
                break;
            }
        }
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
    id: u64,
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

    pub fn is_downloading(&self) -> bool {
        self.dlinfo.is_some()
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn start_download(&mut self) {
        self.dlinfo = Some(DownloadInfo::new());
    }
}

#[derive(Debug, Clone)]
pub struct DownloadInfo {
    progress: usize,
    total: usize,
    elapsed: Duration,
}

impl DownloadInfo {
    pub fn new() -> DownloadInfo {
        DownloadInfo {
            progress: 0,
            total: 0,
            elapsed: Duration::new(0, 0),
        }
    }

    pub fn with_total(total: usize) -> DownloadInfo {
        DownloadInfo {
            progress: 0,
            total: total,
            elapsed: Duration::new(0, 0),
        }
    }
}
