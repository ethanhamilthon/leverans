use reqwest::{multipart, Body};
use std::{
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};
use tokio::sync::{watch, Mutex};

use anyhow::{anyhow, Result};
use futures::{Stream, StreamExt};
use shared::{config::MainConfig, docker::DockerService, err, ok};

use crate::{api::API, data::UserData, utils::get_unix_seconds};

pub struct BuildHandle {
    docker: DockerService,
    abs_path: PathBuf,
    remote_platform: Option<String>,
}

impl BuildHandle {
    pub fn new(
        docker: DockerService,
        abs_path: PathBuf,
        remote_platform: Option<String>,
    ) -> Result<Self> {
        ok!(Self {
            docker: docker,
            abs_path,
            remote_platform,
        })
    }
}

pub async fn build_images(bh: BuildHandle, cfg: MainConfig, token: String) -> Result<()> {
    let build_tasks = get_build_tasks(cfg, bh.remote_platform.clone());
    let arc_bh = Arc::new(bh);
    let (tx, rx) = watch::channel(false);
    let mut joined_tasks = vec![];

    for task in &build_tasks.clone() {
        let task = task.clone();
        let bh = arc_bh.clone();
        let rx = rx.clone();
        joined_tasks.push(tokio::spawn(async move {
            print!("🔧 Building app: {}\n", task.app_name);
            let abs_context = bh.abs_path.join(&task.context);
            let mut logs = vec![];
            let mut stream: Pin<Box<dyn Stream<Item = Result<_, _>> + Send>> = bh
                .docker
                .build_image(
                    &task.docker_file_name,
                    &task.tag,
                    &abs_context.to_str().unwrap(),
                    Some(&task.platform),
                )
                .await
                .map_err(|e| {
                    let err_str = e.to_string();
                    logs.push(err_str.clone());
                    (task.app_name.clone(), logs.clone())
                })?;

            while let Some(msg) = stream.next().await {
                if *rx.borrow() {
                    ok!(task.app_name.clone())
                }
                match msg {
                    Ok(msg) => logs.push(msg.stream.unwrap_or("".to_string())),
                    Err(err) => {
                        let err_str = err.to_string();
                        logs.push(err_str.clone());
                        err!((task.app_name.clone(), logs))
                    }
                }
            }
            println!("✔︎ Building Done: {}\n", task.app_name);
            ok!(task.app_name)
        }));
    }

    for task in joined_tasks {
        if let Err((app_name, logs)) = task.await? {
            tx.send(true)?;
            println!("❌ Build Error: {}\n", app_name);
            for log in logs {
                println!("{}", log);
            }
            err!(anyhow!("Error on building app: {}", app_name));
        }
    }

    upload(arc_bh.clone(), build_tasks, token).await?;
    ok!(())
}
pub async fn upload(bh: Arc<BuildHandle>, images: Vec<BuildTask>, token: String) -> Result<()> {
    let bh = Box::leak(Box::new(Arc::clone(&bh)));
    for task in images {
        print!("📤 Uploading image: {}\n", &task.app_name);

        let stream = bh.docker.save_image(&task.tag).await;
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
        println!("✔︎ Upload Done: {}\n", task.app_name);
    }

    ok!(())
}

#[derive(Clone)]
pub struct BuildTask {
    app_name: String,
    docker_file_name: String,
    context: PathBuf,
    tag: String,
    platform: String,
}

fn get_build_tasks(config: MainConfig, remote_platform: Option<String>) -> Vec<BuildTask> {
    let mut build_tasks = Vec::new();
    if let Some(apps) = config.app.as_ref() {
        for (app_name, app_config) in apps {
            build_tasks.push(BuildTask {
                app_name: app_name.clone(),
                docker_file_name: app_config
                    .dockerfile
                    .clone()
                    .unwrap_or("Dockerfile".to_string()),
                context: Path::new(&app_config.context.clone().unwrap_or("./".to_string()))
                    .to_path_buf(),
                tag: format!(
                    "{}-{}-image:{}",
                    config.project,
                    app_name,
                    get_unix_seconds().to_string()
                ),
                platform: remote_platform.clone().unwrap_or("linux/amd64".to_string()),
            })
        }
    }

    build_tasks
}
