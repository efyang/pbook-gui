use hyper::client::*;
use hyper::header::ContentLength;

#[cfg(unix)]
const FILE_SEP: &'static str = "/";

#[cfg(windows)]
const FILE_SEP: &'static str = "\\";


