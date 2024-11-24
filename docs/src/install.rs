use actix_web::{HttpResponse, Responder, Result};

const INSTALL_CLI_SCRIPT: &str = r#"
#!/bin/bash
echo "Installing Lev CLI..."
"#;

pub async fn install_cli_script(_req: actix_web::HttpRequest) -> Result<impl Responder> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(INSTALL_CLI_SCRIPT))
}

const INSTALL_MANAGER_SCRIPT: &str = r#"
#!/bin/bash
echo "Installing Lev Manager..."
VERSION="$1"
EMAIL="$2"

if [[ -z "$VERSION" ]]; then
  echo "Usage: $0 <version>"
  exit 1
fi

if [[ -z "$EMAIL" ]]; then
  echo "Usage: $0 <version> <email>"
  exit 1
fi

if ! command -v docker &> /dev/null; then
    echo "Installing Docker..."
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    rm get-docker.sh
fi

if ! sudo systemctl is-active --quiet docker; then
    echo "Starting Docker..."
    sudo systemctl start docker
fi

if ! docker info | grep -q 'Swarm: active'; then
    echo "Initializing Docker Swarm..."
    docker swarm init
fi

if [ -z "$(docker network ls --filter name=^lev$ --filter driver=overlay -q)" ]; then
    echo "Creating a network 'lev'..."
    docker network create --driver overlay lev
fi

docker service create \
    --name traefik-service \
    --network lev \
    --publish 80:80 \
    --publish 443:443 \
    --publish 8080:8080 \
    --mount type=bind,source=/var/run/docker.sock,target=/var/run/docker.sock \
    --mount type=volume,source=letsencrypt,target=/letsencrypt \
    --label "traefik.enable=false" \
    traefik:v2.10 \
    --api.insecure=true --api.dashboard=true --entryPoints.web.address=":80" \
    --entryPoints.websecure.address=":443" \
    --providers.docker.swarmMode=true \
    --certificatesresolvers.myresolver.acme.tlschallenge=true \
    --certificatesresolvers.myresolver.acme.email=$EMAIL \
    --certificatesresolvers.myresolver.acme.storage=/letsencrypt/acme.json


docker service create \
    --name lev-service \
    --network lev \
    -e "DBPATH=/data/main.db" \
    --label "traefik.enable=true" \
    --label "traefik.http.routers.lev-service.rule=Headers(\`X-LEVERANS-PASS\`, \`true\`)" \
    --label "traefik.http.routers.lev-service.service=lev-service" \
    --label "traefik.http.services.lev-service.loadbalancer.server.port=8081" \
    --label "traefik.http.routers.lev-service.entrypoints=websecure" \
    --label "traefik.http.routers.lev-service.tls.certresolver=letsencrypt" \
    --label "traefik.http.routers.lev-service.tls=true" \
    --label "traefik.http.routers.lev-service.tls.certresolver=myresolver" \
    --mount type=bind,source=/var/run/docker.sock,target=/var/run/docker.sock \
    --mount type=volume,source=levstore,target=/data/ \
   "leverans/manager:$VERSION" 
"#;

pub async fn install_manager(_req: actix_web::HttpRequest) -> Result<impl Responder> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(INSTALL_MANAGER_SCRIPT))
}

const UNINSTALL_MANAGER_SCRIPT: &str = r#"
#!/bin/bash
echo "Uninstalling Lev Manager..."
docker service rm lev-service
docker service rm traefik-service
docker network rm lev
docker volume rm levstore
"#;

pub async fn uninstall_manager(_req: actix_web::HttpRequest) -> Result<impl Responder> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(UNINSTALL_MANAGER_SCRIPT))
}
