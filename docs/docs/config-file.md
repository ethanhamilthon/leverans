---
title: "Config File"
path: "file"
folder: "config"
order: 1
---

# Configuration File - deploy.yaml

**deploy.yaml** - A file where the entire infrastructure of a single project will be described. It can contain several services and application. You can deploy, upgrade and remove your applications by modifying this config file.

This file is a bit like Docker Compose. Since we made it easier for people who have already worked with Docker. But some seemingly similar things may be different, so read the documentation before using it.

## Main structure

in root level of config there are 3 fields: project, apps, services. let's deal with each of them separately.

```yaml
project: project-name

apps:
  app-name:
    domain: example.com
    port: 8080

services:
  service-name:
    image: redis
    port: 6379
    volumes:
      redis-data: /data
```

### Project name

The name of the current project is specified in the project field. It must be _unique_ in the cluster and _can never be changed_. Since the project name is a unique identifier for Leverans, changing it can have unpredictable consequences.

We recommend that you just call it by the name of the project you are working on. For example, the name of a product, website or service.

### Apps

Apps in the context of Leverans means those docker services that will be created from your code. Anything you intend to build using a Dockerfile or nixpacks should be specified here.

For detailed documentation [go here.](/config/apps)

### Services

Unlike Apps, Services in Leverans are those applications that are already in some Docker Registry (like DockerHub). They will be pulled into the cluster during docker service creation.

For detailed documentation [go here.](/config/services)

## Using with Git

If you are already using git to store code, we highly recommend storing the config file along with the code. This allows you to use GitOps practices. Although Leverans supports Rollback, we believe that rolling back the config along with the code and updating is a better solution.
