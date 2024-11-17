---
title: "Quick Start"
path: "quick-start"
folder: "start"
order: 1
---

# Quick Start: Launch first project

## Make sure you have installed Leverans

Before you start, you need to download Leverans to your server and locally.
If you haven't installed it yet, go to our [installation guide.](/start/install)

## Authentication

After installation, you need to connect to the server from the client. To do this, run the command:

```bash
lev auth -a your-ip -u your-username -p your-password

```

Meaning of arguments:

- **-a** - server address, e.g. _312.90.87.112_
- **-u** - create new username, e.g. _linustorvald_
- **-p** - create new password (make it strong), e.g. _s2dIs9oP98_

Now you created a super user. To make sure that everything was successful use the command `lev whoami`.
