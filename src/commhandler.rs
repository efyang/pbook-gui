use download::*;
use std::sync::mpsc::{Sender, Receiver};
use threadpool::ThreadPool;

pub struct CommHandler {
    threadpool: ThreadPool,
    data: Vec<Category>,
}

impl CommHandler{
    pub fn new(basethreads: usize, start_data: Vec<Category>) -> CommHandler {
        CommHandler {
            threadpool: ThreadPool::new(basethreads),
            data: start_data,
        }
    }

    pub fn update(&mut self) {
        unimplemented!()
    }

    pub fn get_gui_channels(&self) -> () {
        unimplemented!()
    }
}
