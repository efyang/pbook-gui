use hyper::client::*;
use hyper::header::ContentLength;
use std::time::Duration;

// #[cfg(unix)]
// const FILE_SEP: &'static str = "/";

// #[cfg(windows)]
// const FILE_SEP: &'static str = "\\";

pub struct Download {
    pub title: String,
    pub url: String,
    pub enabled: bool,
    dlinfo: Option<DownloadInfo>, /* optional depending on whether
                                   * its currently being downloaded */
}

impl Download {
    fn new(title: String, url: String) -> Download {
        Download {
            title: title,
            url: url,
            enabled: false,
            dlinfo: None,
        }
    }

    fn is_downloading(&self) -> bool {
        self.dlinfo.is_some()
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn start_download(&mut self) {
        self.dlinfo = Some(DownloadInfo::new());
    }
}

pub struct DownloadInfo {
    progress: usize,
    total: usize,
    elapsed: Duration,
}

impl DownloadInfo {
    fn new() -> DownloadInfo {
        DownloadInfo {
            progress: 0,
            total: 0,
            elapsed: Duration::new(0, 0),
        }
    }

    fn with_total(total: usize) -> DownloadInfo {
        DownloadInfo {
            progress: 0,
            total: total,
            elapsed: Duration::new(0, 0),
        }
    }
}
