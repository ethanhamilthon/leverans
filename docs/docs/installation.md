---
title: "Installation"
path: "install"
folder: "start"
order: 3
---

# Installation

Leverans consists of two parts: the Server and the CLI. Both of them must be installed for the system to work.

## Server side

Installing server side is essentially running the docker service in a docker swarm cluster.

Requirements:

- Publicly available IP (optional, needed to make your application accessible from the internet. But must be accessible from wherever your CLI is)
- 0.5 vCPU
- 500 MB RAM
- Linux/amd64 or Linux/arm64
- Installed Docker (If not, we will automatically install it)

To install, run this script in the server. The script must be run via sudo if it doesn't work without it.

```bash
curl -sSL https://get.leverans.dev/manager.sh | sudo sh 0.2.0 <your-email>
```

After that, you should in a few minutes you should get a successful installation message and next steps.

## Client side (CLI)

The CLI tool will also be downloaded via a bash/powershell script. Administrator right is mandatory. A binary specifically for your OS and architecture will be downloaded and installed.

Requirements:

- Docker (the script will not download Docker for you)
- Linux/MacOS/Windows (arm64/amd64)
- Nixpacks (optional, install if you need it)

Run this command in the computer where you have the code.

### with Powershell

You can change iwr to whatever you have installed in your device.

```powershell
iwr -useb https://get.leverans.dev/client.ps1 | iex
```

### with Bash

You can change curl to whatever you have installed in your device.

```bash
curl -sSL https://get.leverans.dev/client.sh | sudo sh
```

### Self Compilation

If you want to compile the CLI from source yourself, clone the repository, navigate to the root folder (Leverans folder), and run the command (rustup must be installed):

```bash
cargo build --release -p lev
```

And longer move the binary to where all the binaries on the system are stored or set up a PATH variable.

### Successful installed

Run the command to see if the CLI was successfully installed:

```bash
lev version
```
