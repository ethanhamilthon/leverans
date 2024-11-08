#!/bin/bash

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
    echo "Docker не установлен. Устанавливаю Docker..."
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    rm get-docker.sh
fi

if ! sudo systemctl is-active --quiet docker; then
    echo "Запуск Docker..."
    sudo systemctl start docker
fi

if ! docker info | grep -q 'Swarm: active'; then
    echo "Инициализация Docker Swarm..."
    docker swarm init
fi

if ! docker network ls --filter name=^lev$ --filter driver=overlay -q; then
    echo "Создание overlay сети 'lev'..."
    docker network create --driver overlay lev
fi
docker volume create acme

docker service create \
    --name traefik-service \
    --network lev \
    --publish 80:80 \
    --publish 443:443 \
    --publish 8080:8080 \
    --mount type=bind,source=/var/run/docker.sock,target=/var/run/docker.sock \
    --mount type=volume,source=letsencrypt,target=/letsencrypt \
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

