---
title: "What is Leverans?"
path: "overview"
folder: "start"
order: 1
---

# What is Leverans?

Leverans is an open-source, next-gen deployment tool that simplifies deploying applications on VPS, VDS, and physical servers. With a user-friendly CLI, it lets you manage and deploy applications in just one click.

## How It Works

Spend 15 minutes setting up the components on your server and local machine. Run _lev auth -a [your-ip]_ to link your machine,
and you are ready to go. Deploying is as simple as creating a config file and running _lev deploy_.

Leverans checks the system’s current state and analyzes your config to create a task sequence.
Leverans uses Docker Swarm and Traefik for core operations but includes its own implementations where needed for reliability.

## Why It Matters

Leverans makes deployment simpler and faster, addressing the pain points of manual setups.
It provides the best price/performance balance for DIY deployment on VPS,
avoiding the complexities of Kubernetes and the costs of managed services like Fly.io or Vercel.

## Core Principles

**Safety:** Prevents human errors and keeps services stable, even during failures.

**Simplicity:** Everything that can be automated is automated, leaving only critical decisions up to you.

**Speed:** In fast-paced CI/CD, speed is essential. The faster you ship features, the better you stay ahead of competitors.

## Key Features

Leverans delivers all the essentials of a modern PaaS:

- Zero downtime deployment
- Rollbacks on errors
- Easy-to-use management interface
- Automatic SSL certificates
- Deploy any Docker-based application
- Automated database backups
- No vendor locks

Additional functionality includes:

- Advanced secrets and environment variable management
- Role-based JWT authentication
- Single-file configuration
- Smart update system
- Load balancing across services and servers
