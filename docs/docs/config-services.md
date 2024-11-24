---
title: "Services Configuration"
path: "services"
folder: "config"
order: 4
---

# Services Configuration

The services fields allow you to deploy an application that is already built.

Common fields are skipped, you can find them [here.](/config/common)

And these fields are specific to apps

```yaml
services:
  service-name:
    image: nginx:latest
```

### Service name

Instead of the service-name key, you usually write the name of your service. Just like the project name, it is a unique identifier for Leverans. Changing it will cause the old docker service to be deleted and a new docker service with a new name to be created.

### Image

The only field that is unique to services is image. This is the docker image that your service uses, the image itself must be accessible from your server (locally, on the local network or on the global internet).
