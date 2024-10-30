use std::{
    io::{self, Write},
    time::Duration,
};

use futures_util::StreamExt;
use tokio::time::sleep;

use crate::{docker::DockerService, docker_platform::get_docker_platform};

pub async fn build() {
    let docker = DockerService::new().unwrap();
    let mut lines_to_delete = 0;
    println!("🔧 Building image...");
    io::stdout().flush().unwrap();

    //println!("🔧 Building image...");
    //io::stdout().flush().unwrap();
    let mut stream = docker
        .build_image(
            "Dockerfile",
            "flowm:latest",
            "/Users/ethanmotion/pro/flower",
            Some(get_docker_platform().unwrap().as_str()),
        )
        .await
        .unwrap();

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(msg) => {
                if let Some(text) = msg.stream {
                    print!("{}", &text);
                    io::stdout().flush().unwrap();
                    lines_to_delete += count_newlines(&text);
                    if lines_to_delete == 10 {
                        clear_lines(10);
                        lines_to_delete = 0
                    }
                }
            }
            Err(_) => {
                println!("😶‍🌫️ Building failed!, lines: {}", lines_to_delete);
                break;
            }
        }
    }

    clear_lines(lines_to_delete);
    println!("🚀 Building done!, lines: {}", lines_to_delete);
    io::stdout().flush().unwrap();
}

pub async fn clear() {
    println!("some string");
    io::stdout().flush().unwrap();

    sleep(Duration::from_millis(2000)).await;

    delete_prev_line();
    println!("Done!");
    io::stdout().flush().unwrap();
}
fn count_newlines(s: &str) -> usize {
    let count = s.chars().filter(|&c| c == '\n').count();
    if count == 0 {
        0
    } else {
        count
    }
}

pub fn clear_lines(n: usize) {
    for _ in 0..n {
        delete_prev_line();
        io::stdout().flush().unwrap();
    }
}

fn delete_prev_line() {
    print!("\x1b[A\x1b[2K");
}

#[tokio::test]
async fn clear_test() {
    build().await;
}
