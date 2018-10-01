### List of Dockerfiles

#### Dockerfiles in `git clone https://github.com/metacurrency/holochain.git`

* `./Dockerfile`

  > Its useful to have this here in this list for completeness, and for people who want to guarantee they have the lastest master of holochain in their image, rather than relying on dockerhub having the latest version
  * creates an image from the latest master of github.com/metacurrency/holochain

* `./Dockerfile.coreDevelopment`
  > This is used for people who are developing the core code. Containers built on this image will allow the developer to manually interact with their new hc / bs servers from the command line
  * uses as a base image the result of `./Dockerfile`
  * splices the current `./ -r` over the top of the base image, and runs `make`, `make bs` and `make test`
  * will fail on build if `make test` fails


#### Dockerfiles in `git clone https://github.com/<MY_PROJECT>/<MY_HOLOCHAIN_DNA>.git`
  > built from a skeleton which includes Dockerfiles

* `./Dockerfile.seedService` && `./Scripts/service.chain.seed`

  > This is used by the docker-compose.yml file to create an a seeded holochain from the DNA in the local filesystem
  * splices the current holochain DNA into the docker image
  * runs `hc clone` and `hc test` for the DNA
  * runs `hc seed <MY_HOLOCHAIN_DNA>`
  * makes the sedded holochain available on a docker volume to all the `./Dockerfile.serveInstances`

* `./Dockerfile.serveInstance` && `./Scripts/chain.joinAndServe`

  > This is used by the docker-compose.yml file to create an instance(s) of `hc server <MY_HOLOCHAIN>`
  * does `hc init <UNIQUE_CONTAINER_ID>`
  * does `hc join <MY_SEEDED_HOLOCHAIN>`
  * exposes the port 3141 for the hc web server to a **random** port(s) on the host machine. docker should be queried to determine the port(s)
  
* multi agent testing

  > we need to come up with a protocol for tests that require more than one hc serve instance. Once that is done, we can create a script / service which implements it

# Overall rationale
* Dockerfiles are designed to enable builds to be fast and regular. Any update to any source code requires a new docker build. build time for the docker image is the same as the time to compile the code and run the tests.
* hc uses ~/.holochain on *your local machine*, as normal, as if there was no docker container. Holochain's that somehow store data outside the mechanisms built into hc need to persist stuff inside their own .holochain/<MY_HOLOCHAIN> directory.

#### Dockerfile devleopers
##### Caveats
* The user and source envs are managed by a script /usr/local/bin/entrypoint.sh.addHostUserToContainer
  * if you need to add your own entry point, make sure to copy this script into the top of your entry point, or else know what you are doing


[dayzee core dev](dayzeecoredev)