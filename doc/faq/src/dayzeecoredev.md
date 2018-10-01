`FROM ubuntu`
> Lets change this to a minimal golang supporting image, wrt ensuring everything works, and the debugging tool `delve` works

`MAINTAINER Duke Dorje && DayZee`
> sorry if this is rude to Duke! xxxx

```
RUN apt-get update
RUN apt-get install -y build-essential software-properties-common python curl wget git-core 

RUN wget -q https://storage.googleapis.com/golang/go1.8.linux-amd64.tar.gz -O golang.tar.gz
RUN tar -zxvf golang.tar.gz -C /usr/local/
RUN mkdir /golang
ENV GOPATH /golang
ENV PATH $GOPATH/bin:/usr/local/go/bin:$PATH

RUN go get -v -u github.com/whyrusleeping/gx
RUN rm $GOPATH/src/github.com/ethereum/go-ethereum/tests -rf
```
> ethereum is planned to be removed as a dependency from this
> AT THIS POINT in the image chain, we have a functioning box with go, and gx

`RUN  go get -v -d github.com/metacurrency/holochain`
> AT THIS POINT: WE HAVE THE LATEST MASTER FROM METACURRENCY/HOLOCHAIN FROM GITHUB

`WORKDIR $GOPATH/src/github.com/metacurrency/holochain`
`RUN make deps`
> AT THIS POINT (CALL IT POINT `DEPS`): we have all (most) of the dependencies, as defined by the laster MASTER branch on GITHUB

`ADD .  $GOPATH/src/github.com/metacurrency/holochain`
> AT THIS POINT: we slice in whatever the current state of our completely independent files on our local machine. So if we have checked out "dev", then we get dev. If we are on our own branch, we get our own branch. Each time we change any of the files under this path, `docker build` will pick that up and run a build starting from the image DEPS and moving forward.

`RUN make`
> This could potentially install or update further dependencies changed since `DEPS` as defined by the make in the current github master. This happens rarely.

> This builds the current version of hc, as defined in the source code of the current state of your development files

`RUN make bs`
> this builds the current version of bs, as defined in the source code of the current state of your development files

`RUN make test`
> this runs the tests, as defined by, and onto the source code of the current state of your development files.

Once this is all complete, we have a new image. This image can be run in a container, and fiddled with directly
OR
used as a base image (see `FROM` at the top of this file) for the FROM of a holochain development cycle.

### All the words in this are relevant