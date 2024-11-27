---
title: "lev auth/login command"
path: "auth-login"
folder: "cli"
order: 3
---

# lev auth/login command

## lev auth

This command is needed to create the first user - super user. A super user is a user who has all accesses in the cluster.

**Flags:**

- `--address or -a` - the remote address of the server (IP address)
- `--username or -u` - create new username
- `--password or -p` - create new password (make it strong)
- `--skip-confirm or -s` - skip password confirmation

## lev login

This is a command to log in to an existing account.

**Flags:**

- `--address or -a` - the remote address of the server (IP address)
- `--username or -u` - your username
- `--password or -p` - your password (make it strong)
- `--skip-confirm or -s` - skip password confirmation
