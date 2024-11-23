use reqwest::{multipart, Body};
use scopeguard::defer;
use std::{
    io::{stdout, Write},
    path::{Path, PathBuf},
    pin::Pin,
    process::{Command, Stdio},
    sync::Arc,
};
use tokio::sync::{watch, Mutex};

use anyhow::{anyhow, Result};
use futures::{Stream, StreamExt};
use shared::{
    config::MainConfig,
    console::new_loader,
    deployable::deploy::{Deploy, DeployTask},
    docker::DockerService,
    err, ok,
};

use crate::{api::API, data::UserData};

pub struct BuildParams {
    pub docker: DockerService,
    pub abs_path: PathBuf,
    pub remote_platform: Option<String>,
    pub main_config: MainConfig,
    pub token: String,
    pub filter: Option<String>,
}

pub async fn new_build_images(
    deploys: Vec<Deploy>,
    abs_path: PathBuf,
    docker: DockerService,
) -> Result<Vec<String>> {
    let build_tasks = deploys.iter().fold(vec![], |mut a, b| {
        b.client_tasks.iter().for_each(|task| {
            if let DeployTask::Build(b) = task {
                a.push(b.clone());
            }
        });
        a
    });
    let (tx, rx) = watch::channel(false);
    let mut joined_tasks = vec![];

    let mut app_names = vec![];
    for task in &build_tasks.clone() {
        let task = task.clone();
        app_names.push(task.tag.clone());
        let abs_context = abs_path.join(&task.context);
        let rx = rx.clone();
        let docker = docker.clone();
        joined_tasks.push(tokio::spawn(async move {
            // dbg!(&task);
            let loader = new_loader(format!(
                "building {} with {}",
                task.short_name,
                if task.is_nix { "nix" } else { "docker" }
            ));
            defer! {
                loader.finish()
            }
            let mut logs = vec![];
            if task.is_nix {
                // dbg!(&task);
                if task.nix_cmds.len() < 3 {
                    logs.push("Nix commands should be at least 3 parts".to_string());
                    err!((task.short_name.clone(), logs))
                }
                let child = Command::new(task.nix_cmds[0].clone())
                    .args(&task.nix_cmds[1..])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| {
                        let err_str = e.to_string();
                        logs.push(err_str.clone());
                        (task.short_name.clone(), logs.clone())
                    })?;

                let output = child
                    .wait_with_output()
                    .map_err(|e| {
                        let err_str = e.to_string();
                        logs.push(err_str.clone());
                        (task.short_name.clone(), logs.clone())
                    })?
                    .stdout;
                let output = String::from_utf8(output).map_err(|e| {
                    let err_str = e.to_string();
                    logs.push(err_str.clone());
                    (task.short_name.clone(), logs.clone())
                })?;
                logs.push(output);
                ok!(task.short_name.clone())
            } else {
                let mut stream: Pin<Box<dyn Stream<Item = Result<_, _>> + Send>> = docker
                    .build_image(
                        &task.docker_file_name,
                        &task.tag,
                        &abs_context.to_str().unwrap(),
                        Some(&task.platform),
                        task.build_args.clone(),
                    )
                    .await
                    .map_err(|e| {
                        let err_str = e.to_string();
                        logs.push(err_str.clone());
                        (task.short_name.clone(), logs.clone())
                    })?;

                while let Some(msg) = stream.next().await {
                    if *rx.borrow() {
                        ok!(task.short_name.clone())
                    }
                    match msg {
                        Ok(msg) => {
                            let text = msg.stream.unwrap_or("".to_string());
                            // println!("{}", text);
                            logs.push(text);
                        }
                        Err(err) => {
                            let err_str = err.to_string();
                            // println!("{}", err_str);
                            logs.push(err_str.clone());
                            err!((task.short_name.clone(), logs))
                        }
                    }
                }
            }
            loader.finish_with_message(format!("built: {}", task.short_name));
            ok!(task.short_name)
        }));
    }

    for task in joined_tasks {
        if let Err((app_name, logs)) = task.await? {
            tx.send(true)?;
            println!("Build Error: {}\n", app_name);
            for log in logs {
                println!("{}", log);
            }
            err!(anyhow!("Error on building app: {}", app_name));
        }
    }
    ok!(app_names)
}

pub async fn upload_images(
    docker: DockerService,
    images: Vec<String>,
    token: String,
) -> Result<()> {
    for task in images {
        let loader = new_loader(format!("uploading {}", task));
        defer! {
            loader.finish()
        }
        let stream = docker.clone().save_image(task.clone());
        let stream_body = Body::wrap_stream(stream);

        let part = multipart::Part::stream(stream_body).file_name("image.tar");

        let form = multipart::Form::new().part("file", part);

        let remote_url = UserData::load_db(false)
            .await?
            .load_current_user()
            .await?
            .remote_url;
        API::new(&remote_url)?
            .upload_image(form, token.clone())
            .await?;
        loader.finish_with_message(format!("uploaded: {}", task));
    }

    ok!(())
}
