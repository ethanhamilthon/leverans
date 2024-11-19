use std::sync::Arc;

use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse, Responder, Result};
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
    let (mut tx, rx) = mpsc::channel(10);

    let handle = tokio::task::spawn_local(async move {
        while let Some(field) = payload.next().await {
            match field {
                Ok(mut field) => {
                    if let Some(content_disp) = field.content_disposition() {
                        println!("Field name: {:?}", content_disp.get_name());
                        println!("Filename: {:?}", content_disp.get_filename());
                    }

                    let mut field_size = 0;
                    while let Some(chunk) = field.next().await {
                        match chunk {
                            Ok(data) => {
                                field_size += data.len();
                                println!("Received chunk size: {} bytes", data.len());
                                if tx.send(data).await.is_err() {
                                    println!("Failed to send chunk through channel");
                                    return;
                                }
                            }
                            Err(e) => {
                                println!("Error reading chunk: {:?}", e);
                                return;
                            }
                        }
                    }
                    println!("Total field size: {} bytes", field_size);
                }
                Err(e) => {
                    println!("Error getting field: {:?}", e);
                    return;
                }
            }
        }
    });

    println!("Starting Docker image load");
    match sv.docker_service.load_image(rx).await {
        Ok(_) => {
            handle.await.unwrap();
            println!("Image load completed successfully");
            Ok(HttpResponse::Ok().body("Image uploaded and loaded successfully"))
        }
        Err(e) => {
            println!("Docker load error: {:?}", e);
            Ok(HttpResponse::InternalServerError().body(format!("Failed to load image: {:?}", e)))
        }
    }
}
