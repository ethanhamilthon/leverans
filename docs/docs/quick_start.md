---
title: "Quick Start"
path: "quick-start"
folder: "start"
order: 4
---

# Quick Start: Launch first project

Before you start, you need to download Leverans to your server and locally. If you haven't installed it yet, go to our [installation guide.](/start/install)

## Authentication

You need to connect to the server from the client, run the command:

```bash
lev auth -a your-ip -u your-username -p your-password
```

Meaning of arguments:

- **-a** - server address, e.g. _312.90.87.112_
- **-u** - create new username, e.g. _linustorvald_
- **-p** - create new password (make it strong), e.g. _s2dIs9oP98_

Now you created a super user. To make sure that everything was successful use `lev whoami`.

## Initialize project and config

Go to the root folder of your project, and run:

```bash
lev new [your-project-name]
```

Now a file named _deploy.yaml_ has been created in the root of your project. If it was not created, you can create it manually.

## Configuration and Deployment

First, let's change the file for the base deployment of our application. You can read all the options of the [configuration file here.](/config/file)

```yaml
project: my-awesome-project

apps:
  backend:
    domain: api.mydomain.com
    port: 8080
```

Here we have created a configuration for a single application. You must have a Dockerfile next to the deploy.yaml file. Since Leverans uses docker to deploy your application.

Now the last step is to execute the command:

```bash
lev deploy
```

And that's it! Within a couple of minutes the application will be finished building and deployed to the Leverans cluster.

After successful deployment you can go to _api.mydomain.com_ to check if everything went well.

## Updating and Creating a Database

Now let's imagine that we decided to use a database in our application. Let's choose the most popular one - Postgresql.

Now we need to do two things: Create a database and connect to our application. Let's modify our config file:

```yaml
project: my-awesome-project

apps:
  backend:
    domain: api.mydomain.com
    port: 8080
    envs:
      DATABASE_URL: "postgres://my-user:my-password@maindb:5432/my-db"

services:
  maindb:
    image: postgres:latest
    volumes:
      pgdata: /var/lib/postgresql/data
    envs:
      POSTGRES_PASSWORD: my-password
      POSTGRES_USER: my-user
      POSTGRES_DB: my-db
```

As you can see, another field, _services_, has appeared on the level with _apps_.

Now once again you just have to execute the command:

```bash
lev deploy
```

And that's it, now we have an application running in a production environment that works with the postgesql database. All this is managed with Docker Swarm and Leverans Manager. In the future you can simply create new features for your application, add new applications/databases/services to the current project and so on.

## So what's next?

- Check our examples in [this repo](https://github.com/ethanhamilthon/leverans/tree/master/examples)
- Check the configuration docs [here](/config/file)
