## use cases
1. core developer
2. app developer
3. multi-node testing (same as app developer as far as I can tell)
4. end-user / package manager

### details
1. core developer
  * build and test current source code
  * potentially run debugger inside container, accessible from the outside world
    * Ive made this work. It does... required go to be installed on the host system, so.. lol
2. app developer
  * build a test current source code
  * integration / multi-node testing
  * deploy?
3. multi-node testing
  * this has been referred to as "headless" (I believe), but I cant see any functional differences between the requirements for this, and for any other dockerfiles
4. end-user / package manager
  * core is a docker image
  * app is a docker image which is mounted into the code
    * since the app is not compiled code, the docker image is simply a docker image of the filesystem
    * since the image hash is a hash of the content, this is sufficient for security purposes