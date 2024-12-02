#!/bin/bash
echo "Uninstalling Lev Manager..."
docker service rm lev-service
docker service rm traefik-service
docker volume rm levstore
