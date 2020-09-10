// This module uses the standard `log` crate.
use log::{debug, error, info, trace, warn};
use std::thread;

pub fn error() {
    error!("using_log {}", 1);
}

pub fn warn() {
    warn!("using_log {}", 1);
}

pub fn info() {
    info!("using_log {}", 1);
}

pub fn debug() {
    debug!("using_log {}", 1);
}

pub fn trace() {
    trace!("using_log {}", 1);
}

pub fn info_in_thread() {
    thread::spawn(|| {
        info!("using_log in thread {}", 1);
    })
    .join()
    .unwrap();
}
