---
title: "Quick Start"
path: "quick-start"
folder: "start"
order: 1
---

# Quick Start: Launch first project

## Make sure you have installed Leverans

Before you start, you need to download Leverans to your server and locally.
If you haven't installed it yet, go to our installation guide.

## Authentication

After installation, you need to connect to the server from the client. To do this, run the command:

```bash
lev auth -a your-ip -u your-username -p your-password

```

Meaning of arguments:

- **-a** - server address, e.g. `312.90.87.112`
- **-u** - create new username, e.g. `linustorvald`
- **-p** - create new password (make it strong), e.g. `s2dIs9oP98`

Now you created a super user. To make sure that everything was successful use the command `lev whoami`.
