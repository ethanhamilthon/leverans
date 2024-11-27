---
title: "All Commands in CLI"
path: "all-commands"
folder: "cli"
order: 1
---

# All Commands in CLI

Since the Leverans cluster is managed in the client side, we need a CLI tool to manage it. But don't worry, you will only use one command 90% of the time - `lev deploy`.

But for different situations, it is good to know other commands. They may solve your problem

Below will be all commands and a short description

### lev deploy

The most important command in the system. We have created a separate page for it in the documentation, [go here.](/cli/deploy)

### lev auth / lev login

Since authentication and login are very important in Leverans, we have also created a separate page in the documentation, [go here.](/cli/auth)

### lev whoami

Command to check if CLI is connected to the server. Command to find out what IP the server has and what our username is

### lev version

Command to check the version of the CLI

### lev new

Command to create a new project, usage example: `lev new my-project`

### lev rollback

Command to rollback the project to the last version. Use in extreme situations. Since Rollback changes the system state regardless of the config file. If possible, we advise to rollback to the previous version of the code and execute `lev deploy`.

**Flags:**

- `--file` - name of the config file to use. If not specified, it will use the default config file (deploy.yaml).
- `--context` - the context of the rollback, it is the name of the project. If not specified, it will use the current context.
- `--skip-confirm` - skip planning confirmation before rollback
- `--unfold` - shows the planning data
- `--timeout` - Timeout on a request to the server. Default is 120 seconds.

All flags support the short version

### lev plan

This is the first part of what you get in `lev deploy`. But unlike `lev plan` it allows you to safely know what will happen in the Leverans cluster on upgrade. Uses the same flags as `lev deploy`

### lev secret

It's a command to manage secrets

**Subcommands:**

- `create` - create a new secret
- `ls` - list all secrets
- `update` - update the secret
- `delete` - delete the secret
- `show` - get the secret

**Flags:**

- `--key or -k` - the key of the secret
- `--value or -v` - the value of the secret

### lev user

This command allows you to create new user, and you can get all users on the system. This command is still in experimental phase, it will be stable soon.

**Subcommands:**

- `create` - create a new user
- `ls` - list all users
