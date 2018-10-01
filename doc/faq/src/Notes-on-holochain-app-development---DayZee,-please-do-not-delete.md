
## Running, Testing and Distributing your app
Holochain Apps can be run inside Docker containers in a production environment. There are always pros and cons however:
* `Holochain Apps` cannot produce unsecure behaviour on the host machine through "root exploits".
* Distribution of your `app` will not require reference to installation of the holochain `core`, as the docker build system will take care of this.
* In situations where users have `apps` which require different versions of the `core`, this will be hidden by the docker tools.

### Running the app
1. [Install Docker](Docker-Installation-for-Developers)
2. 
3. 
    

    $ # spin up a docker container from the image tagged "myholochainapp"
    $ docker run -Pdt myholochainapp
    ```


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

    * This shows that our new container has our holochain app accessible through port `32934` on our host machine (e.g. `http://localhost:32934`)p -r
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



## Play with or test your app
### Run the app to play with
1. Use docker to create a test image of your App and the Core together
## Build developer images

2. Create a developer image of your app to use for testing.

    ```bash
    $ #build a docker image of the app
    $ #  give that docker image the tag "myholochainapp"
    $ #  use the current directory '.' as the context
    $ docker build -t myholochainapp .
    ```
    > **What do I have?**<br><br>
    > The docker image created contains:
    > * a small distribution of linux, called "Alpine"
    > * the Go programming language
    > * all the Go libraries that the Holochain Core depends on
    > * the Holochain Core
    > * and finally, your myholochainapp

    > **What did it do?**
    > * if you did not have them already, it downloaded the Alpine image from dockerhub along with all the Go dependencies from github.
    > * added the latest version of your source files from your host machine (this always happens)
    > * ran hc test myholochainapp
    >   * all the .json files in the /test/ directory of your source code are used to run tests on the code. This means that the code inside the docker image passes its own unit tests.

    > **What next?**<br><br>
    > this is a *developer* image of your app. There are two more stages required for the image to be ready for distribution 

3. Use docker to create a runtime instance of your developer image, called a `docker container`
    
    ```bash
    $ #spin up a container of myholochainapp
    $ #  -P give your network access to your app
    $ #  -d run the container like a service (daemonised)
    $ #  -t use the image with the tag myholochainapp (remember!)
    $ docker run -Pd -t myholochainapp
    ```

    > **What do I have?**
    > * currently running on your computer, a type of tiny virtual machine called a container
    > * the container is running a copy of your app:exclamation:
    > * there is a port (a random port, which we can discover in point 2. below) on the host machine which is connected to port 3141 inside the container. Your app is there! Opening your browser to point at bertha will show the UI for your app. Cool :cool:

    > **What don't I have?**
    > * security vulnerabilities through docker to your host machine

    > **What did it do?**
    > * Some very very very clever things indeed. For now it is important to know that unless you explicitly delete it, the current state (of the hard disk, not of the memory) of every container you ever run, is saved on your host machine. This has two important impacts. One, you can never lose anything stored in a docker container unless you explicitly delete it, and two, eventually you might have to prune the unused containers from your machine. We have scripts to help with this.

2. Determine what port on the host machine connects to your app's UI.
    ```bash
    $ docker ps --latest
    ```

    ```bash
    dbb55a7828b7        myholochainapp        "Scripts/chain.clo..."   6 minutes ago       Up 6 minutes        0.0.0.0:32934->3141/tcp   agitated_brahmagupta