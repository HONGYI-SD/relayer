use std::{thread, time};

pub fn sleep_milliseconds(milliseconds: u64) {
    let millis = time::Duration::from_millis(milliseconds);
    thread::sleep(millis);
}

pub fn sleep_seconds(seconds: u64) {
    let secs = time::Duration::from_secs(seconds);
    thread::sleep(secs);
}
