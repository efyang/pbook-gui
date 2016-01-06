use data::*;
use std::sync::mpsc::{channel, Sender, Receiver, SendError};
use std::collections::HashMap;
use std::thread;
use threadpool::ThreadPool;
use download::*;

const GUI_UPDATE_TIME: usize = 10;

enum DownloadUpdate {
    Message(String),
    Amount(usize),
}

type TpoolProgressMsg = (u64, DownloadUpdate);

pub struct CommHandler {
    threadpool: ThreadPool,
    free_threads: isize,
    data: Vec<Download>,
    jobs: Vec<Download>,
    datacache: HashMap<u64, usize>,
    // sends the new downloads for the gui to update
    // since downloads are the only thing being updated
    gui_update_send: Sender<Vec<Download>>,
    gui_cmd_recv: Receiver<(String, Option<u64>)>,
    // dlid, optional string if error message
    // how to determine whether a thread is done?
    threadpool_progress_recv: Receiver<TpoolProgressMsg>,
    threadpool_progress_send: Sender<TpoolProgressMsg>,
    threadpool_cmd_send: Vec<Sender<(String, Option<u64>)>>,
}

impl CommHandler {
    pub fn new(basethreads: usize,
               start_data: Vec<Download>,
               guichannels: (Sender<Vec<Download>>, Receiver<(String, Option<u64>)>))
               -> CommHandler {
        let (progress_s, progress_r) = channel();
        CommHandler {
            threadpool: ThreadPool::new(basethreads),
            free_threads: basethreads as isize,
            data: start_data.clone(),
            // jobs: Vec::new(),
            jobs: start_data,
            datacache: HashMap::new(),
            gui_update_send: guichannels.0,
            gui_cmd_recv: guichannels.1,
            threadpool_progress_recv: progress_r,
            threadpool_progress_send: progress_s,
            threadpool_cmd_send: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        // handle gui cmd
        match self.gui_cmd_recv.try_recv() {
            Ok(cmd) => {}
            Err(e) => {}
        }

        // handle threadpool message
        match self.threadpool_progress_recv.try_recv() {
            Ok(dl_progress) => {
                self.handle_progress_msg(dl_progress);
            }
            Err(e) => {}
        }

        // start execution of any jobs that exist
        if !self.jobs.is_empty() && self.free_threads > 0 {
            let job = self.jobs.pop().unwrap();
            let progress_sender = self.threadpool_progress_send.clone();
            let (tchan_cmd_s, tchan_cmd_r) = channel();
            self.threadpool_cmd_send.push(tchan_cmd_s);

            self.threadpool.execute(move || {
                for _ in 0..1000000 {
                    progress_sender.send((job.id, DownloadUpdate::Amount(1))).unwrap();
                    // need sleep or there will be a memory overflow -> read more than one byte?
                    thread::sleep_ms(2);
                }
            });
            self.free_threads -= 1;
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
        self.free_threads += 1;
    }

    fn broadcast(&self,
                 msg: (String, Option<u64>))
                 -> Result<(), SendError<(String, Option<u64>)>> {
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
