use std::{sync::Arc, time::Instant};

use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse},
    error::{self, ErrorUnauthorized},
    middleware::{from_fn, Next},
    web, App, Error, HttpResponse, HttpServer,
};
use anyhow::anyhow;
use auth_handler::{handle_is_super_user_exists, login_user, register_super_user};
use deploy_handler::deploy_handle;
use docker_handler::{list_images, upload};
use futures::FutureExt;
use futures_util::future::{ready, Ready};
use healthz_handler::handle_healthz;
use shared::docker::DockerService;

use crate::repo::Repo;

pub mod auth_handler;
pub mod deploy_handler;
pub mod docker_handler;
pub mod healthz_handler;

#[derive(Debug, Clone)]
pub struct ServerData {
    port: u16,
    docker_service: DockerService,
    pub repo: Repo,
}

impl ServerData {
    pub async fn new(port: u16) -> ServerData {
        dbg!("Starting server on port: {}", port);
        ServerData {
            port,
            docker_service: DockerService::new().unwrap(),
            repo: Repo::new(":memory:", true).await.unwrap(),
        }
    }
}

pub async fn start_server(server: ServerData) -> std::io::Result<()> {
    let sv = Arc::new(server);
    let port = sv.port;
    HttpServer::new(move || {
        let server = Arc::clone(&sv);
        App::new()
            .wrap_fn(|req, srv| {
                let now = Instant::now();
                let method = req.method().clone();
                let path = req.path().to_string();
                srv.call(req).map(move |res| {
                    let duration = now.elapsed();
                    if let Ok(ref res) = res {
                        println!("{} {} {} {:?}", method, path, res.status(), duration);
                    }
                    res
                })
            })
            .app_data(web::Data::new(server))
            .route("/upload_image", web::post().to(upload))
            .route("/deploy", web::post().to(deploy_handle))
            .route("/list_images", web::get().to(list_images))
            .route("/healthz", web::get().to(handle_healthz))
            .route("/auth/super", web::get().to(handle_is_super_user_exists))
            .route("/register/super", web::post().to(register_super_user))
            .route("/login/super", web::post().to(login_user))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
