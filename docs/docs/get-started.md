---
title: "Get Started"
path: "get-started"
folder: "start"
order: 1
---

# Deploy Anything to the Internet in One Click

Leverans is a deployment tool that makes launching applications and services to the web straightforward and fast. It automates the heavy lifting while keeping you in full control.

**Powered by Docker, uses Traefik as a reverse proxy, built with Rust.**

## What Makes Leverans Different?

Unlike other open-source solutions (Coolify, Kamal, Dokploy, etc.):

- No vendor lock. Can be used with Docker manually.
- Deploy anything, any way you want.
- No need for SSH, RCE, DockerHub, or Git.
- CLI-based, with all configurations in a single file.
- Deploy or update entire projects with one command.
- Works on any server with minimal requirements (0.5 vCPU, 500 MB RAM).

## Documentation

- Docs: [https://leverans.dev](https://leverans.dev)
- Examples: [https://github.com/ethanhamilthon/leverans/tree/main/examples](https://github.com/ethanhamilthon/leverans/tree/main/examples)

## Installation

Leverans includes two components:

### 1. Server – Install on a server with a public IP for production or locally for testing.

Linux (amd64 or arm64)

```bash
curl -sSL "https://leverans.dev/manager.sh?e=your-email@gmail.com" | sudo sh
```

### 2. CLI – Install where your code resides. Supports Linux, macOS, and Windows on amd64 and arm64.

Linux/amd64, MacOS/amd64 or MacOS/arm64

```bash
curl -sSL "https://leverans.dev/install" | sudo sh
```

Windows/amd64 or Windows/arm64

```powershell
iwr -useb "https://leverans.dev/install?os=win" | iex
```

## Quick Start

Get started fast with clear, easy-to-follow guides in [our documentation.](https://leverans.dev/start/quick-start)

## Version

**Current Version: 0.2.0**

This is the first stable release. Big updates are planned, with version 0.5.0 aiming to deliver a complete, fully refined solution.
