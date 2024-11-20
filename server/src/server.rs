use std::{sync::Arc, time::Instant};

use actix_web::{dev::Service, web, App, HttpServer};
use auth_handler::{
    create_new_user, handle_is_super_user_exists, login_user, register_super_user, user_list,
};
use deploy_handler::handle_deploy;
use docker_handler::upload;
use futures::FutureExt;
use healthz_handler::handle_healthz;
use plan_handler::handle_plan;
use secret_handler::{
    handle_add_secret, handle_delete_secret, handle_list_secrets, handle_update_secret,
};
use shared::docker::DockerService;

use crate::repo::Repo;

pub mod auth_handler;
pub mod deploy_handler;
pub mod docker_handler;
pub mod healthz_handler;
pub mod plan_handler;
pub mod secret_handler;

#[derive(Debug, Clone)]
pub struct ServerData {
    port: u16,
    docker_service: DockerService,
    pub repo: Repo,
}

impl ServerData {
    pub async fn new(port: u16) -> ServerData {
        dbg!("Starting server on port: {}", port);
        let dbpath = std::env::var("DBPATH").unwrap();
        ServerData {
            port,
            docker_service: DockerService::new().unwrap(),
            repo: Repo::new(&dbpath, false).await.unwrap(),
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
            .route("/new-deploy", web::post().to(handle_deploy))
            .route("/plan", web::get().to(handle_plan))
            .route("/healthz", web::get().to(handle_healthz))
            .route("/auth/super", web::get().to(handle_is_super_user_exists))
            .route("/register/super", web::post().to(register_super_user))
            .route("/login/super", web::post().to(login_user))
            .route("/secret", web::post().to(handle_add_secret))
            .route("/secret", web::get().to(handle_list_secrets))
            .route("/secret", web::delete().to(handle_delete_secret))
            .route("/secret", web::put().to(handle_update_secret))
            .route("/users", web::post().to(create_new_user))
            .route("/users", web::get().to(user_list))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
