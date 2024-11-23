# Welcome to Leverans!

- 📖 [Leverans Docs](https://docs.leverans.dev)

## Go + Nixpacks example

This example uses Go and Nixpacks to deploy to a Leverans cluster. Nixpacks is a tool for creating docker images, without a Dockerfile. To use Nixpacks you must have it installed on your computer.

This config is an example of a one-time services in the Leverans cluster. Which can be very useful when creating backups, or other things that need to be done every time you upgrade.

## Deployment with Leverans

1. Open the deploy.yaml file, and change the domain fields to your domains.
2. Run the command `lev deploy`.

And that's it! Now go to your domain, within a few seconds you should see a working website
