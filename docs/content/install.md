# Install

Leverans is made up of two parts. The first must be installed in your remote server,
and the second in your local machine where the application is being developed.

## Install to the Server

First, make sure your server meets these requirements:

- Linux amd64/arm64
- Min. 1 vCPU, 1 GB RAM
- At least has 1 public IP
- Public 80, 433 ports

If everything matches, run the following command in the server:

```bash
sudo curl -L https://install.leverans.dev | bash
```

Wait until all processes have been executed, after that you should see a message about
the success of the installation. There will also be instructions for further action.
