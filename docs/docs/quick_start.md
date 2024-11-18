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

## Initialize project and config

Navigate to the root folder of your project, and run the command:

```bash
lev new [your-project-name]
```

Now a file named _deploy.yaml_ has been created in the root of your project.
If for some reason it was not created, you can create it manually.

## Configuration and Deployment

First, let's change the file for the base deployment of our application.
You can read all the options of the [configuration file here.](/config/file)

```yaml
project: my-awesome-project

apps:
  backend:
    domain: api.mydomain.com
    port: 8080
```

Here we have created a configuration for a single application.
You must have a Dockerfile next to the deploy.yaml file.
Since Leverans uses docker to deploy your application.

In the configuration we have specified:

- _project : my-awesome-project_ - the name of our project, [more details here](/config/file)
- _backend_ - the name of our application ( both names should never change, so choose the name carefully ).
- _domain: api.mydomain.com_ - the domain from which our application will be accessible. The domain must be [connected to the server.](/server/domain)
- _port: 8080_ - the port on which our application is running, since we have a Go project, the default port is 8080.

Now the last step is to execute the command:

```bash
lev deploy
```

If you have done everything correctly you should see 2 Tasks: the first one is Build, the second one is Create.
Enter `y` to move on. And that's it! Within a couple of minutes the application will be finished building and
deployed to the Leverans cluster.

After successful deployment you can go to _api.mydomain.com_ to check if everything went well.

## Updating and Creating a Database

Now let's imagine that we decided to use a database in our beautiful application.
We can of course use any database we want, since Leverans allows us to host any application that runs on Docker.

But Leverans has a couple of options [already configured](/databases/ready-to-go). Let's choose the most popular one - Postgresql.

Now we need to do two things: Create a database and connect to it from our application. Let's modify our config file:

```yaml
project: my-awesome-project

apps:
  backend:
    domain: api.mydomain.com
    port: 8080
    envs:
      DATABASE_URL: "{{ this.maindb.connection }}"

databases:
  maindb:
    from: pg
```

As you can see, another field, _databases_, has appeared on the level with _apps_.
And there with 2 lines of code we created a new postgresql database.

And since our application needs the address of connection to the database,
we passed there env variable `DATABASE_URL`. Here we used a thing called [Smart String](/concept/smart-strings).
In short, it's just a way of telling Leverans: “Hey, put a string there to connect to the maindb database”.

Now once again you just have to execute the command:

```bash
lev deploy
```

And that's it, now we have an application running in a production environment that works with the postgesql database.
All this is managed with Docker Swarm and Leverans Manager. In the future you can simply create new features for your application,
add new applications/databases/services to the current project and so on.

## So what's next?

- Check our examples in [this repo](https://github.com/ethanhamilthon/leverans/tree/master/examples)
- Check the configuration docs [here](/config/file)
