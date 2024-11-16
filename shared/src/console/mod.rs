use anyhow::{anyhow, Result};
use std::{
    io::{stdin, stdout, Write},
    time::Duration,
};

use indicatif::{ProgressBar, ProgressStyle};

use crate::ok;

pub fn new_loader(loading_msg: String) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠸", "⠴", "⠦", "⠇", "✔"]),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message(loading_msg);

    spinner
}

pub fn ask(question: &str) -> Result<String> {
    print!("{}", question);
    stdout().flush()?;
    let mut answer = String::new();
    stdin()
        .read_line(&mut answer)
        .map_err(|e| anyhow!("Error on reading username: {}", e))?;
    ok!(answer.trim().to_string())
}

#[test]
fn test_loader() {
    let l = new_loader("uploading...".into());
    std::thread::sleep(Duration::from_millis(5000));
    l.finish_with_message("done");
    assert!(true);
}

