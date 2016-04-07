use data::*;
use std::sync::mpsc::{channel, Sender, Receiver, SendError};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};
use threadpool::ThreadPool;
use downloader::*;
use helper::{Ignore, name_to_fname, name_to_dname};
use gui::update_gui;
use time::precise_time_ns;
use constants::GUI_UPDATE_TIME;
use std::fs::create_dir_all;
use std::ffi::OsStr;
use fsthread::*;

pub struct CommHandler {
    threadpool: ThreadPool,
    max_threads: Arc<Mutex<usize>>,
    current_threads: Arc<Mutex<usize>>,
    // the current model
    // id:download
    data: HashMap<u64, Download>,
    // list of all ids
    current_ids: Vec<u64>,
    // jobs: Vec<Download>,
    jobs: VecDeque<Download>,
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
    fsthread_send: Sender<FsCommand>,
    fsthread_recv: Receiver<FsUpdate>,
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
            let (fsthread_send, fsthread_recv) = FsThread::spawn();
            CommHandler {
                threadpool: ThreadPool::new(basethreads),
                max_threads: Arc::new(Mutex::new(basethreads)),
                current_threads: Arc::new(Mutex::new(0)),
                data: id_data_hm,
                current_ids: Vec::new(),
                jobs: VecDeque::new(), // FIFO queue
                datacache: HashMap::new(),
                pending_changes: Vec::new(),
                gui_update_send: guichannels.0,
                gui_cmd_recv: guichannels.1,
                threadpool_progress_recv: progress_r,
                threadpool_progress_send: progress_s,
                fsthread_send: fsthread_send,
                fsthread_recv: fsthread_recv,
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
            let job = self.jobs.pop_front().unwrap();
            let progress_sender = self.threadpool_progress_send.clone();
            let (tchan_cmd_s, tchan_cmd_r) = channel();
            self.threadpool_cmd_send.push(tchan_cmd_s);
            let mut downloader = Downloader::new(job, tchan_cmd_r, progress_sender);
            {
                *self.current_threads.lock().unwrap() += 1;
                let current_threads = self.current_threads.clone();
                self.threadpool.execute(move || {
                    let mut keep_downloading = true;
                    match downloader.begin() {
                        Ok(_) => {}
                        Err(e) => {
                            *current_threads.lock().unwrap() -= 1;
                            keep_downloading = false;
                            match &e as &str {
                                "finished" => {}
                                _ => {
                                    downloader.send_panicked(e.to_owned());
                                }
                            }
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
                                        downloader.send_panicked(e.to_owned());
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
            for idx in 0..self.current_ids.len() {
                let id = self.current_ids[idx];
                let mut download = self.data.get_mut(&id).unwrap();
                if download.downloading() {
                    // do not add a change if it is not actually downloading - unnecessary update
                    download.increment_progress(*self.datacache.get(&id).unwrap_or(&0))
                        .unwrap();
                    if download.progress().unwrap_or(0) > 0 && !download.finished() {
                        // add to pending changes
                        self.pending_changes.push(GuiChange::Set(idx, download.to_owned()));
                    }
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

        // check fsthread updates
        match self.fsthread_recv.try_recv() {
            Ok(update) => self.handle_fsthread_update(update),
            Err(_) => {}
        }
    }

    fn handle_gui_cmd(&mut self, cmd: GuiCmdMsg) {
        match cmd {
            GuiCmdMsg::Add(id, path) => {
                let mut download = self.data.get_mut(&id).unwrap();
                download.start_download();
                download.set_enable_state(true);
                download.set_path(path);
                // add to jobs
                self.jobs.push_back(download.clone());
                self.current_ids.push(id);
                // add to pending changes
                self.pending_changes.push(GuiChange::Add(download.to_owned()));
            }
            GuiCmdMsg::Restart(id, path) => {
                let mut download = self.data.get_mut(&id).unwrap();
                download.start_download();
                download.set_enable_state(true);
                download.set_path(path);
                self.jobs.push_front(download.clone());
                let idx = self.current_ids.iter().position(|&x| x == id).unwrap();
                self.pending_changes.push(GuiChange::Set(idx, download.to_owned()));
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

                {
                    let mut dl = self.data.get_mut(&id).unwrap();
                    dl.set_enable_state(false);
                    dl.stop_download();
                }

                // add to pending changes
                for idx in 0..self.current_ids.len() {
                    if self.current_ids[idx] == id {
                        self.current_ids.remove(idx);
                        self.pending_changes.push(GuiChange::Remove(idx));
                        break;
                    }
                }
                // broadcast to all threads
                if !in_jobs {
                    self.broadcast(TpoolCmdMsg::Remove(id)).ignore();
                }
            }
            GuiCmdMsg::ChangeDir(newdir) => {
                // Copy over all of the finished downloads
                for id in self.current_ids.iter() {
                    let mut dl = self.data.get_mut(id).unwrap();
                    if dl.finished() {
                        let fname = name_to_fname(dl.name());
                        let oldpath = dl.path().join(fname.to_owned());
                        let newpath;
                        if let Some(ref category_name) = dl.category_name() {
                            let category_dir = newdir.to_owned()
                                .join(name_to_dname(category_name));
                            create_dir_all(&category_dir).expect("Failed to create dir");
                            newpath = category_dir.join(fname);
                        } else {
                            newpath = newdir.to_owned()
                                .join(fname);
                        }
                        // if the copy is failed then it has already been renamed; ignore this
                        // TODO: make this use coroutines/threads to minimize update lag
                        self.fsthread_send
                            .send(FsCommand::Copy(oldpath.clone(), newpath.clone()))
                            .expect("FsThread send fail");
                        // copy(oldpath.clone(), newpath.clone())
                        // .expect(&format!("Copy fail {:?} -> {:?} Download: {:#?}",
                        // oldpath.clone(),
                        // newpath.clone(),
                        // dl));
                        self.fsthread_send
                            .send(FsCommand::Remove(oldpath))
                            .expect("FsThread send fail");
                        // remove_file(oldpath).expect("Remove fail");
                    }
                    dl.set_path(newdir.to_owned());
                }
                // change all of the pending jobs to the new path
                for job in self.jobs.iter_mut() {
                    let current_path = job.path();
                    let category_name = job.category_name();
                    let backup_fname = name_to_fname(job.name());
                    let current_fname = current_path.file_name()
                        .unwrap_or(OsStr::new(&backup_fname));
                    let newpath;
                    if let Some(category) = category_name {
                        newpath = newdir.to_owned()
                            .join(name_to_dname(&category))
                            .join(current_fname);
                    } else {
                        newpath = newdir.to_owned().join(current_fname);
                    }
                    job.set_path(newpath);
                }
                // broadcast to downloaders
                self.broadcast(TpoolCmdMsg::ChangeDir(newdir)).ignore();
            }
            GuiCmdMsg::Stop => {
                self.broadcast(TpoolCmdMsg::Stop).ignore();
                self.fsthread_send.send(FsCommand::Stop).ignore();
                drop(self)
            }
        }
    }

    fn handle_progress_msg(&mut self, progress: TpoolProgressMsg) {
        let id = progress.0;
        match progress.1 {
            DownloadUpdate::SetSize(content_length) => {
                let mut download = self.data.get_mut(&id).unwrap();
                download.set_total(content_length);
                for idx in 0..self.current_ids.len() {
                    if self.current_ids[idx] == id {
                        self.pending_changes.push(GuiChange::Set(idx, download.to_owned()));
                        break;
                    }
                }
            }
            DownloadUpdate::Amount(amount) => {
                // add to cache
                self.datacache.increment(id, amount);
            }
            DownloadUpdate::Finished => {
                let mut download = self.data.get_mut(&id).unwrap();
                download.set_finished();
                for idx in 0..self.current_ids.len() {
                    if self.current_ids[idx] == id {
                        // remove any other sets
                        for i in (0..self.pending_changes.len()).rev() {
                            if let GuiChange::Set(otheridx, _) = self.pending_changes[i] {
                                if otheridx == idx {
                                    self.pending_changes.remove(i);
                                }
                            }
                        }
                        self.pending_changes.push(GuiChange::Set(idx, download.to_owned()));
                        break;
                    }
                }
            }
            DownloadUpdate::Message(msg) => {
                println!("{}", msg);
            }
            DownloadUpdate::Panicked(error) => {
                let ref download = self.data[&id];
                let mut newerr = download.name().to_owned() + ": ";
                newerr.push_str(&error);
                self.pending_changes.push(GuiChange::Panicked(true, newerr));
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

    fn handle_fsthread_update(&mut self, update: FsUpdate) {
        match update {
            FsUpdate::Error(msg) => {
                self.pending_changes.push(GuiChange::Panicked(true, format!("FsThread error: {}", msg)));
            },
        }
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
