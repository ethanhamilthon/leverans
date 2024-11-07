use anyhow::{anyhow, Result};
use shared::ok;
use std::{
    fs,
    io::{Read, Write},
    thread::{sleep, sleep_ms},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub fn open_file_as_string(path: &str) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    ok!(contents)
}

pub fn get_unix_seconds() -> u64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn create_home_file(file_path: &str) -> Result<String> {
    let mut path = dirs::home_dir().ok_or(anyhow!("Failed to get home dir"))?;
    path.push(file_path);

    let dir_path = path.parent().ok_or(anyhow!("Failed to get parent dir"))?;

    fs::create_dir_all(dir_path)?;

    if !path.exists() {
        fs::File::create(&path)?.write_all(b"")?;
    }
    Ok(path.to_str().unwrap().to_owned())
}

#[test]
fn test_unix_seconds() {
    let seconds = get_unix_seconds();
    assert_ne!(seconds, 0);

    sleep(Duration::from_secs(1));
    let seconds2 = get_unix_seconds();
    assert!(seconds != seconds2);
}
