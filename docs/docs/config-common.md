---
title: "Common fields"
path: "common"
folder: "config"
order: 2
---

# Common fields

Here will be those fields of apps and services that are common to each of them. We decided not to duplicate their descriptions to make it more convenient

All common fields:

```yaml
project: project-name

apps: # or services
  app-name: # any name of app or service
    domain: example.com
    port: 8080
    path-prefix: /api
    expose: [8080]
    envs:
      ENV1: value1
    labels:
      label1: value1
    volumes:
      some-volume: /data/some
    mounts:
      /etc/data: /data/some
    args: ["--flag1", "--flag2"]
    cmds: ["cmd1", "cmd2"]
    replicas: 2
    constraints: ["node.role == manager"]
    cpu: 0.5
    memory: 512
    https: true
    proxy:
      - domain: another.example.com
        port: 8081
    health-check:
      cmd: ["CMD-SHELL", "your health check command"]
      interval: 10
      timeout: 5
      retries: 3
      start-period: 5
    restart: always
```

### Domain

We use the Domain field to create a reverse proxy in front of your application. Here you need to specify the domain name from which your application will be accessed. The domain name itself should already be connected to the server

### Port

Mandatory field if you are going to use domain. Here you should specify the port your application is listening on. Even if you don't want your application to be open to the global internet, specify the port so that you can use Smart Strings.

### Path-prefix

Just as domain is a routing rule for reverse proxy. If you specify it, all requests to that domain and that start with that path prefix will be redirected to your application

For example:
If you specify `path-prefix: /api` all requests to example.com/api will go to that application and example.com/dashboard for example will go to other applications.

### Expose

Those ports that will be accessible from the outside. By default all applications except reverse proxy (80, 443 ports) are not available outside the docker network. And only applications running in the current cluster can access each other. But for some reason if you need to open a port on the host, use expose. It takes an array of numbers.

### Evironment variables

Adds env variables on containers in runtime. Must be specified without `-`

### Labels

Custom labels for your container. By default, Leverans puts labels for Traefik there. So instead of writing labels for Traefik yourself, use domain, port, path-prefix. Labels are only needed when you have complex routing logic or need them for other services (e.g. swarm-cronjob).

### Volumes

Docker volumes for your application. They are automatically created if they were not found by default settings of the docker itself. It is used for long term data storage for services (for example, for a database). For storing important data, implement backup mechanisms.

### Mounts

Used to retrieve data from the Host machine. It is not recommended for using data storage as Volumes.

### Args

Arguments that will be added to the commands specified in the docker image, or to the commands specified in the `cmds` field.

### Cmds

Overrides the CMD that was specified in the docker image.

### Replicas

The number of running copies for the service. By default it is 2 for apps and 1 for services. 2 copies for apps are needed to have zero downtime updates when the application is updated. But if your application is statefull, you can set 1 too.

### Constraints

Constraints for the placement of the service. By default, it is empty.

### CPU

The number of CPU cores allocated to the container. By default, it is 1

### Memory

The amount of memory allocated to the container. By default, it is 1024 MB

### HTTPS

Whether to use HTTPS or not. By default, it is true. You can use false for the local Leverans cluster to avoid browser errors.

### Proxy

Same fields as domain, port, path-prefix. But allows to define several rules for routing.

### Health-check

You need to specify a health checking rule. By chance it is not specified, and it is impossible to know what state your application is in. Specifying a health check gives Docker swarm a finer control, improves application availability and reduces requests for unhealthy copies of the application.

Sub-fields to specify:

#### Cmd

This will be the command to be executed in the container, accepts an array of strings. Don't forget to specify the first element _CMD-SHELL_

#### Interval

The interval at which the health check will be executed. By default, it is 10 seconds.

#### Timeout

The timeout for the health check. By default, it is 5 seconds.

#### Retries

The number of times to retry the health check. By default, it is 3.

#### Start-period

The time to wait before the first health check. By default, it is 5 seconds.

### Restart

Set whether to restart after container completion. Accepts 3 types of value: any (or always), no (or none), on-failure (or failure).

- Any - containers will be restarted no matter what. Even if they completed successfully (with code 0).
- No - containers will be started only 1 time, and will not be restarted again. Useful if you need to run some scripts 1 time in the server.
- On-failure - containers will be restarted only if they failed (not with code 0).
