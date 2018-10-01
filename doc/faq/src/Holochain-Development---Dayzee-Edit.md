# Holochain Development

## Rationale 
The holochain skeleton includes a set of scripts that use docker. The guiding principal is that your personal development tools should be untouched. Installation of docker and docker-compose, add your development user to the `docker` group, and you have complete access to the full suite of development and publising tools available from the Holochain commnunity.

## Toolchain
Please take the time to read this page thoroughly, at least once :), so that it is clear what are the capacities of the toolchain.

### 1. 1



First we will take you through the development process using an example Holochain App DNA from our own repository, and then we will show you how to use a skeleton to start your own project.

## A Simple Working Example with `examples/clutter`

1. Install the ***latest*** version of Docker Compose directly from [the docker website](https://docs.docker.com/compose/install/) - ( You may have to install Docker too)
2. Clone the `metacurrency/holoSkel` repository [from github](https://github.com/metacurrency/holoSkel

    ```bash 
    $ #navigate to where you wanna be
    $ mkdir myHolochainApp
    $ cd myHolochainApp
    $ git clone https://github.com/metacurrency/holoSkel.git .
    ```
3. This will create a skeleton holochain app for you in the current directory (`.`), and also downloads a set of example holochain apps in to `./examples`. `examples/clutter` is a simple decentralised messaging app built on Holochain. Our toolchain uses docker containers to remove the need for you to maintain the holochain software. To run an example of the clutter app:

    ```bash
    $ cd examples/clutter
    $ docker build -t clutter
    $ docker run -Pdt clutter
    ```
    > this means `docker run`:
    * `-P` map all exposed ports onto random ports on the host
    * `-d` run the container in daemonised mode
    * `-t` the image we built, tagged as `clutter` in the line before

    by default, `hc` will serve your holochain on port 3141. Because docker is a container, this is always a good port to use, and docker with `-P` will manage passing through port 3141 to an open port on your host machine. to find out what port your holochain app has been mapped to, run:

    ```bash
    $ docker ps --latest
    ```

    ```bash
    dbb55a7828b7        clutter             "Scripts/chain.clo..."   6 minutes ago       Up 6 minutes        0.0.0.0:32934->3141/tcp   agitated_brahmagupta
    ```

    * This shows that our new container has our holochain app accessible through port `32934` on our host machine (e.g. `http://localhost:32934`)
    * we can also see that the container is called `agitated_brahmagupta` (docker names are good chat)
    * and that the first bunch of characters of the hash of the container are `dbb55a7828b7`

4. We can destroy this container instance using either the name or the hash of the container

    ```bash
    $ docker kill agitated_brahmagupta
    or
    $ docker kill dbb
    or
    $ docker kill agi
    ```

5. docker works very well in a development cycle. Each time you have editted code, and you want to test the new version, run `docker build...`, `docker run...`

6. TODO: If you have implemented tests for your holochain, you will see the output each time you call `docker run...`
    `hc test clutter` needs to be added to the script

7. Distributed apps need to be, well, distributed. Docker makes it very easy for you to test many instances of your app and how they interact with each other. To set up a cluster of nodes, use `docker-compose up` and then `docker-compose scale`

    ```bash
    $ docker-compose up
    $ docker-compose scale hc=2
    $ docker ps --last=2
    ```
    > this means:
    * `scale` how many instances of some service do you want to spin up
    * `hc` is the holochain service. Each one of these runs an instance of your holochain app
    * `scale hc=2` means: make it so there are 2 hc containers running. The number 2 can be replaced with however many instances you would like to have. Remember that to access them from the outside world, there must be a spare port on the host machine to connect to.

8. docker compose makes it easy to take down your containers, and rebuild the images:

    ```bash
    $ docker-compose down
    $ docker-compose build
    $ docker-compose up
    ```

9. TODO check out if docker-compose updates running containers if images are changed????

## The Bootstrap Server
hc instances use our holochain of holochains to self locate onto the network. The (admitedly rather simple!) output of this can be seen at http://bootstrap.holochain.net:10000