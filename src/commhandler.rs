use download::*;
use std::sync::mpsc::{channel, Sender, Receiver};
use threadpool::ThreadPool;

pub struct CommHandler {
    threadpool: ThreadPool,
    data: Vec<Download>,
    jobs: Vec<Download>,
    // sends the new downloads for the gui to update
    // since downloads are the only thing being updated
    gui_update_send: Sender<Vec<Download>>,
    gui_cmd_recv: Receiver<String>,
    // dlid, optional string if error message
    threadpool_progress_recv: Receiver<(u64, Option<String>)>,
    threadpool_cmd_send: Vec<Sender<(String, Option<String>)>>,
}

impl CommHandler{
    pub fn new(basethreads: usize,
               start_data: Vec<Download>,
               guichannels: (Sender<Vec<Download>>, Receiver<String>))
               -> CommHandler {
        let (progress_s, progress_r) = channel();

        CommHandler {
            threadpool: ThreadPool::new(basethreads),
            data: start_data,
            jobs: Vec::new(),
            gui_update_send: guichannels.0,
            gui_cmd_recv: guichannels.1,
            threadpool_progress_recv: progress_r,
            threadpool_cmd_send: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        // handle gui cmd
        match self.gui_cmd_recv.try_recv() {
            Ok(cmd) => {
                
            },
            Err(e) => {
                
            },
        }
        
        // handle threadpool message
        match self.threadpool_progress_recv.try_recv() {
            Ok(dlid) => {
                
            }
            Err(e) => {
                
            },
        }

        // start execution of any jobs that exist
        if !self.jobs.is_empty() {
            
        }
        unimplemented!()
    }
}
