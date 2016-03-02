use data::*;
use std::sync::mpsc::{channel, Sender, Receiver, SendError};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use threadpool::ThreadPool;
use downloader::*;
use std::time::Duration;
use helper::Ignore;
use gui::update_gui;
use time::precise_time_ns;
use constants::GUI_UPDATE_TIME;

pub struct CommHandler {
    threadpool: ThreadPool,
    max_threads: Arc<Mutex<usize>>,
    current_threads: Arc<Mutex<usize>>,
    // the current model
    data: Vec<Download>,
    // id:download
    id_data: HashMap<u64, Download>,
    liststore_ids: Vec<u64>,
    jobs: Vec<Download>,
    // download id, amount of bytes to add
    datacache: HashMap<u64, usize>,
    // pending changes
    pending_changes: GuiUpdateMsg,
    // sends list of changes
    gui_update_send: Sender<GuiUpdateMsg>,
    gui_cmd_recv: Receiver<GuiCmdMsg>,
    // dlid, optional string if error message
    // how to determine whether a thread is done?
    threadpool_progress_recv: Receiver<TpoolProgressMsg>,
    threadpool_progress_send: Sender<TpoolProgressMsg>,
    threadpool_cmd_send: Vec<Sender<TpoolCmdMsg>>,
    next_gui_update_t: u64,
}

impl CommHandler {
    pub fn new(basethreads: usize,
               start_data: Vec<Download>,
               guichannels: (Sender<GuiUpdateMsg>, Receiver<GuiCmdMsg>))
               -> CommHandler {
        let (progress_s, progress_r) = channel();
        let mut id_data_hm = HashMap::new();
        for download in start_data.iter() {
            id_data_hm.insert(download.id(), download.clone());
        }
        CommHandler {
            threadpool: ThreadPool::new(basethreads),
            max_threads: Arc::new(Mutex::new(basethreads)),
            current_threads: Arc::new(Mutex::new(0)),
            data: start_data.clone(),
            id_data: id_data_hm,
            liststore_ids: Vec::new(),
            jobs: Vec::new(),
            datacache: HashMap::new(),
            pending_changes: Vec::new(),
            gui_update_send: guichannels.0,
            gui_cmd_recv: guichannels.1,
            threadpool_progress_recv: progress_r,
            threadpool_progress_send: progress_s,
            threadpool_cmd_send: Vec::new(),
            next_gui_update_t: precise_time_ns() + GUI_UPDATE_TIME,
        }
    }

    pub fn update(&mut self) {
        // handle gui cmd
        match self.gui_cmd_recv.try_recv() {
            Ok(cmd) => self.handle_gui_cmd(cmd),
            Err(_) => {
                // No message in queue
            }
        }

        // handle threadpool messages
        let mut messages_clear = false;
        while !messages_clear {
            if precise_time_ns() >= self.next_gui_update_t {
                messages_clear = true;
            }
            match self.threadpool_progress_recv.try_recv() {
                Ok(dl_progress) => {
                    self.handle_progress_msg(dl_progress);
                }
                Err(_) => {
                    messages_clear = true;
                }
            }
        }


        // start execution of any jobs that exist
        let max_threads = self.max_threads.lock().unwrap().clone();
        let current_threads = self.current_threads.lock().unwrap().clone();
        if !self.jobs.is_empty() && (max_threads - current_threads) > 0 {
            let job = self.jobs.pop().unwrap();
            let progress_sender = self.threadpool_progress_send.clone();
            let (tchan_cmd_s, tchan_cmd_r) = channel();
            self.threadpool_cmd_send.push(tchan_cmd_s);
            let mut downloader = Downloader::new(job, tchan_cmd_r, progress_sender);
            {
                *self.current_threads.lock().unwrap() += 1;
                let max_threads = self.max_threads.clone();
                let current_threads = self.current_threads.clone();
                self.threadpool.execute(move || {
                    let mut keep_downloading = true;
                    match downloader.begin() {
                        Ok(_) => {}
                        Err(e) => {
                            *current_threads.lock().unwrap() -= 1;
                            panic!(e);
                        }
                    }
                    while keep_downloading {
                        match downloader.update() {
                            Ok(_) => {}
                            Err(e) => {
                                *current_threads.lock().unwrap() -= 1;
                                keep_downloading = false;
                                match &e as &str {
                                    "finished" | "stopped" => {}
                                    _ => {
                                        println!("{}", e);
                                    }
                                }
                            }
                        }
                    }
                    drop(downloader);
                });
            }
        }

        // update the gui
        let current_time = precise_time_ns();
        if current_time >= self.next_gui_update_t {
            // add everything from the datacache to the main data
            let mut idx = 0;
            for download in self.data.iter_mut() {
                let id = download.id();
                if download.downloading() {
                    download.increment_progress(*self.datacache.get(&id).unwrap_or(&0))
                            .unwrap();
                    // add to pending changes
                    self.pending_changes.push(GuiChange::Set(idx, download.to_owned()));
                    idx += 1;
                }
            }
            // clear datacache
            self.datacache.clear();
            // send the changes
            if let Err(e) = self.gui_update_send.send(self.pending_changes.to_owned()) {
                if e.description() != "sending on a closed channel" {
                    println!("Failed to send gui update message: {}", e);
                }
            }
            // clear pending changes
            self.pending_changes.clear();
            self.next_gui_update_t = current_time + GUI_UPDATE_TIME;
            update_gui()
        }
    }

    fn handle_gui_cmd(&mut self, cmd: GuiCmdMsg) {
        match cmd {
            GuiCmdMsg::Add(id, path) => {
                let mut download = self.id_data[&id].clone();
                download.start_download();
                download.set_enable_state(true);
                download.set_path(path);
                // add to jobs
                self.jobs.push(download.clone());
                self.liststore_ids.push(id);
                // add to pending changes
                self.pending_changes.push(GuiChange::Add(download.to_owned()));
                // start in main data model
                for item in self.data.iter_mut() {
                    if item.id() == id {
                        item.set_enable_state(true);
                        item.start_download();
                        break;
                    }
                }
            }
            GuiCmdMsg::Remove(id) => {
                let mut in_jobs = false;
                // remove from jobs if existing
                for idx in 0..self.jobs.len() {
                    if self.jobs[idx].id() == id {
                        self.jobs.remove(idx);
                        in_jobs = true;
                        break;
                    }
                }
                // remove from main data model
                for idx in 0..self.data.len() {
                    let ref mut dl = self.data[idx];
                    if dl.id() == id {
                        dl.set_enable_state(false);
                        dl.stop_download();
                        break;
                    }
                }

                // add to pending changes
                for idx in 0..self.liststore_ids.len() {
                    if self.liststore_ids[idx] == id {
                        self.liststore_ids.remove(idx);
                        self.pending_changes.push(GuiChange::Remove(idx));
                        break;
                    }
                }
                // broadcast to all threads
                if !in_jobs {
                    self.broadcast(TpoolCmdMsg::Remove(id)).ignore();
                }
            }
            GuiCmdMsg::Stop => {
                self.broadcast(TpoolCmdMsg::Stop).ignore();
                drop(self)
            }
        }
    }

    fn handle_progress_msg(&mut self, progress: TpoolProgressMsg) {
        let dlid = progress.0;
        match progress.1 {
            DownloadUpdate::SetSize(content_length) => {
                let mut idx = 0;
                for download in self.data.iter_mut() {
                    if &download.id() == &dlid {
                        download.set_total(content_length);
                        break;
                    }
                    if download.downloading() {
                        idx += 1;
                    }
                }
                self.id_data
                    .get_mut(&dlid)
                    .expect("No such id_data entry")
                    .set_total(content_length);
            }
            DownloadUpdate::Amount(dlamnt) => {
                // add to cache
                self.datacache.increment(dlid, dlamnt);
            }
            DownloadUpdate::Message(message) => {
                // work on message handling
                match &message as &str {
                    "finished" => {
                        // get idx
                        let mut idx = 0;
                        for download in self.data.iter_mut() {
                            if &download.id() == &dlid {
                                break;
                            }
                            if download.downloading() {
                                idx += 1;
                            }
                        }
                        // send message to gui
                        self.pending_changes.push(GuiChange::Finished(idx));
                    }
                    _ => {}
                }
            }
        }
    }

    fn broadcast(&self, msg: TpoolCmdMsg) -> Result<(), SendError<TpoolCmdMsg>> {
        for channel in self.threadpool_cmd_send.iter() {
            let sendresult = channel.send(msg.clone());
            if sendresult.is_err() {
                return sendresult;
            }
        }
        Ok(())
    }
}

trait AutoIncrement {
    fn increment(&mut self, key: u64, value: usize);
}

impl AutoIncrement for HashMap<u64, usize> {
    fn increment(&mut self, key: u64, value: usize) {
        let current = self.entry(key).or_insert(0);
        *current += value;
    }
}
