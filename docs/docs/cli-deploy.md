---
title: "lev deploy command"
path: "deploy"
folder: "cli"
order: 2
---

# lev deploy command

This is probably the most commonly used command at Leverans. It deploys the application to the cluster from your code and the deploy.yaml config.

This team does four things at once:

- Planning
- Building the Docker image
- Upload the image to the server
- Deployment

**Flags:**

- `--skip-confirm` - skip planning confirmation before deployment
- `--unfold` - shows the planning data
- `--timeout` - Timeout on a request to the server. Default is 120 seconds.
- `--context` - the folder where is deploy.yaml file. If not specified, it will use the current context.
- `--file` - the name of the config file. If not specified, it will use the default deploy.yaml file.
- `--build` - Specifies which applications to build, if _build_ field in config is _manual_.

**Filtering:**
Filtering is a feature in Leverans that allows you to deploy specifically one or more applications while ignoring the rest of the update.

- `filter` subcommand - deploys only 1 application. Using `lev deploy app-name`
- `--only` flag - deploys multiple application. Usage `lev deploy --only app-name1,app-name2,app-name3`
