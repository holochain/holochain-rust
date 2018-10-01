# Docker Usage Notes
We have set up the docker environment such that the ~/.holochain directory in the containers are mapped to your user directory.  This allows that any chains you are working on will be maintained across container builds, and additionally that you can access the same chains from both docker containers and `hc` installed on your machine.

## Helpful Docker Commands
 - `docker info` - displays information about your current docker system configuration
 - `docker run <container-name>` - launches the named container
 - `docker ps -a` - lists operational containers whether they're running or terminated
 - `docker rm <container-hash-id>` - removes operational container from memory
 - `docker start <container-hash-id>` - starts a container which had been stopped
 - `docker attach <container-hash-id>` - attaches your terminal session to that container

## Helpful Keyboard Shortcuts
 - `<container-hash-id>` - just type the first few characters of one of the docker IDs and press <TAB> to auto-complete it.
 - `Ctrl-D` - exits and terminates running docker container
 - `Ctrl-P Ctrl-Q` - detaches from container leaving it running

## Cheat Sheet
 - Check out this [cheat-sheet](https://github.com/wsargent/docker-cheat-sheet) for tons more help making your way around docker