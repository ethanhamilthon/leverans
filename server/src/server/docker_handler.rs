use std::sync::Arc;

use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse, Responder, Result};
use bytes::Bytes;
use futures::{channel::mpsc, SinkExt, StreamExt};

use crate::{repo::user_repo::RoleType, server::auth_handler::must_auth};

use super::ServerData;

pub async fn upload(
    sv: web::Data<Arc<ServerData>>,
    mut payload: Multipart,
    req: HttpRequest,
) -> Result<impl Responder> {
    println!("upload");
    must_auth(
        &req,
        vec![
            RoleType::FullAccess,
            RoleType::SuperUser,
            RoleType::UpdateOnly,
        ],
    )?;
    let images_dir = std::env::var("IMAGES_DIR").unwrap_or("/images".to_string());

    std::fs::create_dir_all(&images_dir).expect("Failed to create images directory");

    while let Some(field) = payload.next().await {
        match field {
            Ok(mut field) => {
                if let Some(content_disp) = field.content_disposition() {
                    if let Some(filename) = content_disp.get_filename() {
                        let file_path = format!("{}/{}", &images_dir, filename);

                        let mut file = match tokio::fs::File::create(&file_path).await {
                            Ok(f) => f,
                            Err(e) => {
                                println!("Failed to create file: {:?}", e);
                                return Ok(
                                    HttpResponse::InternalServerError().body("Failed to save file")
                                );
                            }
                        };

                        while let Some(chunk) = field.next().await {
                            match chunk {
                                Ok(data) => {
                                    if tokio::io::AsyncWriteExt::write_all(&mut file, &data)
                                        .await
                                        .is_err()
                                    {
                                        println!("Failed to write to file");
                                        return Ok(HttpResponse::InternalServerError()
                                            .body("Failed to save file"));
                                    }
                                }
                                Err(e) => {
                                    println!("Error reading chunk: {:?}", e);
                                    return Ok(HttpResponse::InternalServerError()
                                        .body("Failed to process file"));
                                }
                            }
                        }

                        drop(file);

                        println!("File saved successfully: {}", file_path);
                        println!("Loading image from file: {}", file_path);

                        let file_stream = match tokio::fs::File::open(&file_path).await {
                            Ok(f) => tokio_util::io::ReaderStream::new(f)
                                .map(|result| result.map(Bytes::from).unwrap()),
                            Err(e) => {
                                println!("Failed to open saved file: {:?}", e);
                                return Ok(HttpResponse::InternalServerError()
                                    .body("Failed to open saved file"));
                            }
                        };

                        if let Err(e) = sv.docker_service.load_image(file_stream).await {
                            println!("Docker load error: {:?}", e);
                            return Ok(HttpResponse::InternalServerError()
                                .body(format!("Failed to load image: {:?}", e)));
                        }

                        println!("Image loaded successfully from file: {}", file_path);

                        tokio::fs::remove_file(&file_path).await.unwrap();
                    }
                }
            }
            Err(e) => {
                println!("Error getting field: {:?}", e);
                return Ok(
                    HttpResponse::InternalServerError().body("Failed to process multipart payload")
                );
            }
        }
    }

    Ok(HttpResponse::Ok().body("Image uploaded and loaded successfully"))
}
