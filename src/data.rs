use std::time::Duration;
use std::hash::{Hash, Hasher, SipHasher};
pub use std::path::{Path, PathBuf};
use time::precise_time_s;
use time;
use helper::{minimum, maximum, make_string_if_nonzero};
use constants::DOWNLOAD_SPEED_UPDATE_TIME;
use std::i64;
// refactor TpoolProgressMsg to just be a DownloadUpdate
pub enum DownloadUpdate {
    Message(String),
    Amount(usize),
    SetSize(usize),
    Panicked(String),
    Finished,
}

pub enum GuiCmdMsg {
    Add(u64, PathBuf),
    Restart(usize), // usize = index
    Remove(u64),
    ChangeDir(PathBuf),
    Stop,
}

#[derive(Clone, Debug)]
pub enum TpoolCmdMsg {
    Remove(u64),
    ChangeDir(PathBuf),
    Stop,
}

pub type TpoolProgressMsg = (u64, DownloadUpdate);
pub type GuiCmd = (String, Option<u64>, Option<PathBuf>);

#[derive(Clone)]
#[allow(dead_code)]
pub enum GuiChange {
    Remove(usize), // idx
    Add(Download), // download
    Set(usize, Download), // idx, download
    Panicked(bool, String), // id -- work on this
}

// pub type GuiUpdateMsg = Vec<(String, Option<u64>, Option<usize>, Option<Download>)>;
pub type GuiUpdateMsg = Vec<GuiChange>;

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

    // Getter functions

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn downloads(&self) -> &[Download] {
        &self.downloads
    }

    // pub fn ids(&self) -> Vec<u64> {
    // self.downloads.iter().map(|dl| dl.id()).collect::<Vec<u64>>()
    // }

    pub fn get_download_at_idx(&self, idx: usize) -> &Download {
        &self.downloads[idx]
    }

    pub fn enabled(&self) -> bool {
        self.downloads.iter().all(|x| x.enabled())
    }

    // Setter functions

    pub fn add_download(&mut self, download: Download) {
        self.downloads.push(download);
    }
}

pub fn get_hash_id(name: &str, url: &str) -> u64 {
    let mut hasher = SipHasher::new();
    format!("{}{}", name, url).hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone)]
pub struct Download {
    id: u64,
    name: String,
    url: String,
    enabled: bool,
    download_info: Option<DownloadInfo>, /* optional depending on whether
                                          * its currently being downloaded */
    category_name: Option<String>,
}

impl Download {
    pub fn new(name: &str, url: &str, category_name: Option<String>) -> Download {
        // id is siphash of name + url
        Download {
            id: get_hash_id(name, url),
            name: name.to_owned(),
            url: url.to_owned(),
            enabled: false,
            download_info: None,
            category_name: category_name,
        }
    }

    // Getter functions

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn download_info(&self) -> &Option<DownloadInfo> {
        &self.download_info
    }

    pub fn downloading(&self) -> bool {
        self.download_info.is_some()
    }

    pub fn finished(&self) -> bool {
        if let Some(ref download_info) = self.download_info {
            download_info.finished()
        } else {
            false
        }
    }

    pub fn category_name(&self) -> Option<String> {
        self.category_name.clone()
    }

    pub fn path(&self) -> PathBuf {
        self.clone().download_info.unwrap().get_path()
    }

    // Setter functions

    pub fn set_total(&mut self, total: usize) {
        if let Some(ref mut download_info) = self.download_info {
            download_info.set_total(total);
        }
    }

    pub fn set_path(&mut self, path: PathBuf) {
        if let Some(ref mut download_info) = self.download_info {
            download_info.set_path(path);
        }
    }

    pub fn set_enable_state(&mut self, newstate: bool) {
        self.enabled = newstate;
    }

    pub fn start_download(&mut self) {
        self.download_info = Some(DownloadInfo::new());
    }

    pub fn stop_download(&mut self) {
        self.download_info = None;
    }

    // Incremental functions

    pub fn increment_progress(&mut self, increment: usize) -> Result<(), String> {
        if let Some(ref mut download_info) = self.download_info {
            download_info.increment_progress(increment);
            Ok(())
        } else {
            Err("Progress cannot be incremented because it is not downloading.".to_owned())
        }
    }

    pub fn set_finished(&mut self) {
        if let Some(ref mut download_info) = self.download_info {
            download_info.set_finished();
        }    
    }

    pub fn progress(&self) -> Option<usize> {
        if let Some(ref download_info) = self.download_info {
            Some(download_info.progress())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadInfo {
    failed: bool,
    progress: usize,
    total: usize,
    previous_progress: usize,
    recent_progress: usize,
    recent_progress_clear_time: f64,
    finished: bool,
    elapsed: Duration,
    path: PathBuf,
}

impl DownloadInfo {
    pub fn new() -> DownloadInfo {
        DownloadInfo {
            failed: false,
            progress: 0,
            total: 0,
            previous_progress: 0,
            recent_progress: 0,
            recent_progress_clear_time: precise_time_s() + DOWNLOAD_SPEED_UPDATE_TIME,
            finished: false,
            elapsed: Duration::new(0, 0),
            path: PathBuf::new(),
        }
    }

    // Getters
    
    pub fn progress(&self) -> usize {
        self.progress
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.to_path_buf()
    }

    pub fn percentage(&self) -> f32 {
        minimum(self.progress as f32 / maximum(self.total as f32, 1.0), 1.0)
    }

    // to bytes per second
    pub fn speed(&self) -> f32 {
        self.recent_progress as f32 / DOWNLOAD_SPEED_UPDATE_TIME as f32
    }

    // to seconds
    pub fn eta(&self) -> String {
        let bytes_left;
        if self.total < self.progress {
            bytes_left = 0;
        } else {
            bytes_left = self.total - self.progress;
        }
        let speed = self.speed();
        let eta = bytes_left as f32 / speed;
        let streta;
        if self.progress == 0 && self.total == 0 {
            streta = "N/A".to_owned();
        } else if self.progress >= self.total {
            streta = "Done.".to_owned();
        } else if maximum(eta, 1000000.0) == eta || speed == 0.0 {
            streta = "∞".to_owned();
        } else {
            let dur = time::Duration::seconds(maximum(minimum(eta as i64, i64::MAX), 0));
            streta = format!("{}{}{}{}{}s",
                             make_string_if_nonzero(dur.num_weeks(), "w"),
                             make_string_if_nonzero(dur.num_days(), "d"),
                             make_string_if_nonzero(dur.num_hours(), "h"),
                             make_string_if_nonzero(dur.num_minutes(), "m"),
                             dur.num_seconds());
        }
        streta
    }

    // Setters

    pub fn set_total(&mut self, total: usize) {
        self.total = total;
    }

    pub fn set_finished(&mut self) {
        self.recent_progress = 0;
        self.progress = self.total;
        self.finished = true;
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
    }

    // Incremental functions

    pub fn increment_progress(&mut self, increment: usize) {
        self.progress += increment;
        let timenow = precise_time_s();
        if timenow >= self.recent_progress_clear_time {
            self.previous_progress = self.recent_progress;
            if self.progress == self.total || increment == 0 {
                self.recent_progress /= 2;
            } else {
                self.recent_progress = (self.recent_progress + self.previous_progress) / 2;
            }
            self.recent_progress_clear_time = timenow + DOWNLOAD_SPEED_UPDATE_TIME;
        } else {
            self.recent_progress += increment;
        }
    }
}
