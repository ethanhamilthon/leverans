use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

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

#[test]
fn test_loader() {
    let l = new_loader("uploading...".into());
    std::thread::sleep(Duration::from_millis(5000));
    l.finish_with_message("done");
    assert!(true);
}

