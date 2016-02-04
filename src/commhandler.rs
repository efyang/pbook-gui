use data::*;
use std::sync::mpsc::{channel, Sender, Receiver, SendError};
use std::collections::HashMap;
use std::thread;
use threadpool::ThreadPool;
use download::*;
use std::time::Duration;
use helper::Ignore;
use gui::update_gui;
use time::precise_time_ns;
use constants::GUI_UPDATE_TIME;

pub struct CommHandler {
    threadpool: ThreadPool,
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
               guichannels: (Sender<GuiUpdateMsg>, Receiver<(String, Option<u64>)>))
               -> CommHandler {
        let (progress_s, progress_r) = channel();
        let mut id_data_hm = HashMap::new();
        for download in start_data.iter() {
            id_data_hm.insert(download.get_id(), download.clone());
        }
        CommHandler {
            threadpool: ThreadPool::new(basethreads),
            data: start_data.clone(),
            id_data: id_data_hm,
            liststore_ids: Vec::new(),
            jobs: Vec::new(),
            // jobs: start_data,
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

        // handle threadpool message
        match self.threadpool_progress_recv.try_recv() {
            Ok(dl_progress) => {
                self.handle_progress_msg(dl_progress);
            }
            Err(_) => {}
        }

        // start execution of any jobs that exist
        if !self.jobs.is_empty() &&
           (self.threadpool.max_count() - self.threadpool.active_count()) > 0 {
            let mut job = self.jobs.pop().unwrap();
            job.start_download();
            let progress_sender = self.threadpool_progress_send.clone();
            let (tchan_cmd_s, tchan_cmd_r) = channel();
            self.threadpool_cmd_send.push(tchan_cmd_s);
            let mut downloader = Downloader::new(job,
                                                 tchan_cmd_r,
                                                 progress_sender,
                                                 Path::new("./testing"));
            self.threadpool.execute(move || {
                loop {
                    match downloader.begin() {
                        Ok(_) => {},
                        Err(e) => panic!(e),
                    }
                    // progress_sender.send((job.id, DownloadUpdate::Amount(1))).unwrap();
                    // need sleep or there will be a memory overflow -> read more than one byte?
                    match downloader.update() {
                        Ok(_) => {}
                        Err(e) => panic!(e),
                    }

                    thread::sleep(Duration::from_millis(0));
                }
            });
        }

        // update the gui
        let current_time = precise_time_ns();
        if current_time >= self.next_gui_update_t {
            // add everything from the datacache to the main data
            for download in self.data.iter_mut() {
                let id = download.get_id();
                if download.is_downloading() {
                    download.increment_progress(*self.datacache.get(&id).unwrap_or(&0))
                            .unwrap();
                }
            }
            // clear datacache
            self.datacache.clear();
            // send the changes
            if let Err(e) = self.gui_update_send.send(self.pending_changes.to_owned()) {
                println!("Failed to send gui update message: {}", e);
            }
            // clear pending changes
            self.pending_changes.clear();
            self.next_gui_update_t = current_time + GUI_UPDATE_TIME;
            update_gui()
        }
    }

    fn handle_gui_cmd(&mut self, cmd: GuiCmdMsg) {
        match &cmd.0 as &str {
            "add" => {
                let mut download = self.id_data[&cmd.1.unwrap()].clone();
                let id = download.get_id();
                download.start_download();
                download.set_enable_state(true);
                // add to jobs
                self.jobs.push(download.clone());
                self.liststore_ids.push(download.get_id());
                // add to changes
                self.pending_changes.push((cmd.0, None, None, Some(download.clone())));
                // start in main data model
                for item in self.data.iter_mut() {
                    if item.get_id() == id {
                        item.set_enable_state(true);
                        item.start_download();
                        break;
                    }
                }
            }
            "remove" => {
                let id = cmd.1.unwrap();
                // remove from jobs if existing
                for idx in 0..self.jobs.len() {
                    if self.jobs[idx].get_id() == id {
                        self.jobs.remove(idx);
                        break;
                    }
                }
                // remove from main data model
                for idx in 0..self.data.len() {
                    let ref mut dl = self.data[idx];
                    if dl.get_id() == id {
                        dl.set_enable_state(false);
                        dl.stop_download();
                        break;
                    }
                }

                for idx in 0..self.liststore_ids.len() {
                    if self.liststore_ids[idx] == id {
                        self.liststore_ids.remove(idx);
                        self.pending_changes.push((cmd.0.clone(), None, Some(idx), None));
                        break;
                    }
                }
                // broadcast to all threads
                self.broadcast(cmd).ignore();
                // change to unwrap later on
            }
            "stop" => {
                self.broadcast(cmd).ignore();
                drop(self)
            }
            _ => {}
        }
    }

    fn handle_progress_msg(&mut self, progress: TpoolProgressMsg) {
        let dlid = progress.0;
        match progress.1 {
            DownloadUpdate::Amount(dlamnt) => {
                // add to cache
                self.datacache.increment(dlid, dlamnt);
            }
            DownloadUpdate::Message(message) => {
                // work on message handling
            }
        }
    }

    fn broadcast(&self, msg: TpoolCmdMsg) -> Result<(), SendError<(String, Option<u64>)>> {
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
