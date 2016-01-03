use download::*;
use std::sync::mpsc::{channel, Sender, Receiver};
use threadpool::ThreadPool;

pub struct CommHandler {
    threadpool: ThreadPool,
    data: Vec<Category>,
    gui_update_send: Sender<Vec<Category>>,
    gui_cmd_recv: Receiver<String>,
    threadpool_progress_recv: Receiver<u64>,
    threadpool_cmd_send: Vec<Sender<(String, Option<String>)>>,
}

impl CommHandler{
    pub fn new(basethreads: usize,
               start_data: Vec<Category>,
               guichannels: (Sender<Vec<Category>>, Receiver<String>))
               -> CommHandler {
        let (progress_s, progress_r) = channel();
        CommHandler {
            threadpool: ThreadPool::new(basethreads),
            data: start_data,
            gui_update_send: guichannels.0,
            gui_cmd_recv: guichannels.1,
            threadpool_progress_recv: progress_r,
            threadpool_cmd_send: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        match self.gui_cmd_recv.try_recv() {
            Ok(cmd) => {

            },
            Err(e) => {

            },
        }

        match self.threadpool_progress_recv.try_recv() {
            Ok(dlid) => {
                
            }
            Err(e) => {
                
            },
        }
        unimplemented!()
    }

    fn handle_gui_cmd(&mut self) {
        unimplemented!() 
    }

    fn handle_threadpool_progress(&mut self) {
        unimplemented!()
    }
}
