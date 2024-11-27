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
  echo "Version not specified"
  exit 1
fi

if [[ -z "$EMAIL" ]]; then
  echo "Email not specified."
  exit 1
fi

if ! command -v docker &> /dev/null; then
    echo "Docker not found. Installing Docker..."

    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64)
            PLATFORM="x86_64"
            ;;
        aarch64)
            PLATFORM="aarch64"
            ;;
        *)
            echo "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    DOCKER_VERSION="27.3.1"

    echo "Downloading Docker $DOCKER_VERSION for $PLATFORM..."
    curl -fsSL "https://download.docker.com/linux/static/stable/${PLATFORM}/docker-${DOCKER_VERSION}.tgz" -o docker.tgz

    echo "Extracting Docker binaries..."
    tar xzvf docker.tgz

    echo "Installing Docker binaries..."
    sudo cp docker/* /usr/local/bin/
    sudo chmod +x /usr/local/bin/docker*

    # Настройка systemd службы для Docker
    echo "Configuring Docker service..."
    sudo tee /etc/systemd/system/docker.service > /dev/null <<EOF
[Unit]
Description=Docker Service
After=network.target

[Service]
ExecStart=/usr/local/bin/dockerd
Restart=always
User=root

[Install]
WantedBy=multi-user.target
EOF

    echo "Starting Docker service..."
    sudo systemctl daemon-reload
    sudo systemctl start docker
    sudo systemctl enable docker

    echo "Docker installation complete!"
else
    echo "Docker is already installed."
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
    -e "IMAGES_DIR=/images" \
    --label "traefik.enable=true" \
    --label "traefik.http.routers.lev-service.rule=Headers(\`X-LEVERANS-PASS\`, \`true\`)" \
    --label "traefik.http.routers.lev-service.service=lev-service" \
    --label "traefik.http.services.lev-service.loadbalancer.server.port=8081" \
    --label "traefik.http.routers.lev-service.entrypoints=web" \
    --mount type=bind,source=/var/run/docker.sock,target=/var/run/docker.sock \
    --mount type=volume,source=levstore,target=/data/ \
    --mount type=volume, source=levimage, target=/images/ \
    "leverans/manager:$VERSION"

echo "Lev Manager setup complete!"
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
docker volume rm levstore
"#;

pub async fn uninstall_manager(_req: actix_web::HttpRequest) -> Result<impl Responder> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(UNINSTALL_MANAGER_SCRIPT))
}
