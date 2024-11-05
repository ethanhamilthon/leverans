#!/bin/bash

VERSION="$1"

if [[ -z "$VERSION" ]]; then
  echo "Usage: $0 <version>"
  exit 1
fi

# check if docker is installed
if ! command -v docker &> /dev/null; then
    echo "Docker is not installed. Please install Docker and try again."
    exit 1
fi 

# check if docker buildx is installed
if ! command -v docker buildx &> /dev/null; then
    echo "Docker Buildx is not installed. Please install Docker Buildx and try again."
    exit 1
fi 

# check if login to docker hub is successful
if ! docker login; then
    echo "Docker login failed. Please check your credentials and try again."
    exit 1
fi 

echo "Building Docker image: leverans/manager:$VERSION"
docker buildx build --platform linux/amd64,linux/arm64 -t leverans/manager:$VERSION --push .


