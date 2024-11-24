---
title: "Apps Configuration"
path: "apps"
folder: "config"
order: 3
---

# Apps Configuration

The Apps field usually describes those applications that are created with your code. Therefore, unlike services here you will find several fields that control the process of creating a docker image from your code

Common fields are skipped, you can find them [here.](/config/common)

And these fields are specific to apps

```yaml
apps:
  app-name:
    build: manual # or auto
    builder: docker # or nixpack
    nix-cmds: ["nixpacks", "build", "<context>", "--name", "<tag>"]
    build-args:
      ARG1: value1
    dockerfile: Dockerfile
    context: .
```

### App name

Instead of the app-name key, you usually write the name of your app. Just like the project name, it is a unique identifier for Leverans. Changing it will cause the old docker service to be deleted and a new docker service with a new name to be created.

### Build

The build field has only 2 types of value: _auto_ or _manual_. This field responds to the build of your docker image. That is, if auto is set, every time you execute `lev deploy` a new docker image is created. Which may not be necessary when the code change was only in one application, and the other one didn't change. For this case you can use manual. As long as you don't exactly say `lev deploy --build app-name`, a new docker image will not be created

### Builder

It also accepts only 2 types of values: _docker_ or*nixpacks*. This is used to build your docker image. If the value is docker, the image will be created with Dockerfile. If nixpacks, it will be responsible for building the docker image.

### Nix Commands

If you are using nixpacks as your builder, you will need to specify the command to create the image. But you don't have to specify a command, Leverans will use it: `nixpacks build <context-from-config or ./ > --name <image-name> --platform <target-platform>`. But sometimes you will need to manage nixpacks yourself. To do this, you can specify commands to execute in the nix-cmds field

The important point is that your commands must match:

- You cannot name the docker image yourself. Use the <tag> string instead of the image docker name, Leverans will automatically replace it with the real image name.
- Use the <context> string so where you need to specify the context of the image build.
- The commands should be an array of strings that starts with _nixpacks_

### Build time arguments

buid-args are the arguments that docker accepts when building an application. They are similar in use to env variables.

### Dockerfile

The name of your Dockerfile, the default is simply _Dockerfile_.

### Context

The name of the path to your Dockerfile, the default is _./_
