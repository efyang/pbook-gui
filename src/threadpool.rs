use std::thread::*;

pub struct ThreadPool {
    max_threads: isize,
    open_thread: isize,
    threads: Vec<JoinHandle>,
}
