use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};

use anyhow::{anyhow, Result};
use bollard::{
    image::{BuildImageOptions, CreateImageOptions, ImportImageOptions, ListImagesOptions},
    secret::BuildInfo,
};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::Serialize;
use tar::Builder;
use walkdir::WalkDir;

use crate::{docker_platform::get_docker_platform, ok};

use super::DockerService;

impl DockerService {
    pub async fn pull_image(&self, image_name: &str) -> Result<()> {
        let options = Some(CreateImageOptions {
            from_image: image_name,
            ..Default::default()
        });

        let mut stream = self.conn.create_image(options, None, None);

        println!("Pulling image: {}", image_name);

        while let Some(pull_result) = stream.next().await {
            match pull_result {
                Ok(detail) => {
                    if let Some(status) = detail.status {
                        println!("{}", status);
                    }
                }
                Err(e) => {
                    eprintln!("Error pulling image: {}", e);
                    return Err(anyhow!(format!("error to pull image")));
                }
            }
        }

        println!("Image {} pulled successfully.", image_name);
        Ok(())
    }

    pub async fn list_images(&self) -> Result<Vec<DockerImage>> {
        let opts = Some(ListImagesOptions::<String> {
            all: true,
            ..Default::default()
        });
        let image_summaries = self.conn.list_images(opts).await?;

        let images = image_summaries.iter().fold(Vec::new(), |mut acc, x| {
            x.repo_tags.iter().for_each(|tag| {
                acc.push(DockerImage {
                    tag: tag.clone(),
                    image_id: x.id.clone(),
                })
            });
            acc
        });

        Ok(images)
    }

    pub async fn load_image<T>(&self, stream: T) -> Result<()>
    where
        T: Stream<Item = Bytes> + Send + 'static,
    {
        let load_options = ImportImageOptions { quiet: false };
        let mut result = self.conn.import_image_stream(load_options, stream, None);

        while let Some(s) = result.next().await {
            println!("status {:?}", s?.status);
        }

        Ok(())
    }

    pub fn save_image(
        &self,
        image_name: String,
    ) -> Pin<Box<dyn Stream<Item = Result<Bytes, bollard::errors::Error>> + Send>> {
        Box::pin(self.conn.export_image(&image_name))
    }

    pub async fn build_image(
        &self,
        docker_file_name: &str,
        image_name: &str,
        context: &str,
        platform: Option<&str>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<BuildInfo, bollard::errors::Error>> + '_ + Send>>>
    {
        let options = BuildImageOptions {
            dockerfile: docker_file_name, // Название Dockerfile
            t: image_name,                // Тег для образа
            rm: true,
            platform: platform.unwrap_or("linux/amd64"),
            ..Default::default()
        };

        // Открываем контекст сборки (архивированный контекст или директорию)
        let build_context = Self::create_tar_context(context).await.unwrap();

        // Запускаем сборку образа
        let build_stream = self
            .conn
            .build_image(options, None, Some(build_context.into()));

        ok!(Box::pin(build_stream))
    }

    pub async fn create_tar_context(context_path: &str) -> Result<Vec<u8>> {
        let context_path = Path::new(context_path);
        let dockerignore = Self::load_ignore_list(context_path)?;
        let mut tar_builder = Builder::new(Vec::new());

        for entry in WalkDir::new(context_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let relative_path = path.strip_prefix(context_path)?;

            // Пропускаем файлы, которые соответствуют правилам в .dockerignore
            if dockerignore.iter().any(|ignored| path.starts_with(ignored)) {
                continue;
            }
            // Пропускаем корневую директорию
            if relative_path.as_os_str().is_empty() {
                continue;
            }

            let name = if relative_path.components().count() == 0 {
                PathBuf::from(".")
            } else {
                relative_path.to_path_buf()
            };

            if path.is_file() {
                tar_builder.append_path_with_name(path, name)?;
            } else if path.is_dir() {
                tar_builder.append_dir(name, path)?;
            }
        }

        let tar_data = tar_builder.into_inner()?;
        Ok(tar_data)
    }

    fn load_ignore_list(context_path: &Path) -> std::io::Result<Vec<PathBuf>> {
        let dockerignore_path = context_path.join(".dockerignore");
        let mut ignore_list = Vec::new();

        if dockerignore_path.exists() {
            let file = File::open(dockerignore_path)?;
            for line in BufReader::new(file).lines() {
                let line = line?.trim().to_string();
                if !line.is_empty() && !line.starts_with('#') {
                    ignore_list.push(context_path.join(line));
                }
            }
        }

        Ok(ignore_list)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DockerImage {
    pub image_id: String,
    pub tag: String,
}

#[tokio::test]
#[ignore]
async fn build_image_test() {
    let docker = DockerService::new().unwrap();
    let _ = docker
        .build_image(
            "Dockerfile",
            "flowm:v6",
            "/Users/ethanmotion/pro/flower",
            Some(get_docker_platform().unwrap().as_str()),
        )
        .await
        .unwrap();
}
