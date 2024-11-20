# Deploy anything to Internet in one click

Leverans is a tool that automates the deployment of applications and services to the web.
Helping you deploy everything in a bluntly simpler way, while leaving you in full control.

Works on top of Docker, uses Traefik for reverse proxy, Written in Rust.

Unlike other open source solutions (Coolify, Kamal, Dokploy, etc):

- You can deploy anything you want, any way you want.
- No SSH, No RCE, No DocketHub, No Git required.
- Works via CLI, all project configuration in one config file.
- You can deploy/update the whole project through one command.
- Works on any server ( min requirement: 0.5 vCPU, 500 mb RAM ).

## Documentation

- Docs: [https://docs.leverans.dev](https://docs.leverans.dev)
- Examples: [https://github.com/ethanhamilthon/leverans/tree/main/examples](https://github.com/ethanhamilthon/leverans/tree/main/examples)

## Installation

Leverans consists of 2 components: Server and CLI tool. Both will need to be installed in order for the system to work correctly.

### Server part

To install a server part for production use you need a server with a public IP address (All VPS/VDS providers give it by default).
Also for testing and local use you can install on your local computer, but it will work only from local network.

### CLI part

The CLI tool should be installed where your code is. Supported by Linux, Macos, Windows on amd64 and arm64 architectures.

## Fast start

## Version

The current 0.2.0 is the first stable version.
We have big plans, so there may be a breaking update in the future. In 0.5.0 we are expecting a full-fledged ideaogical project.
