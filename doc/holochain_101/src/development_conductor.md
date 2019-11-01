# Development Conductor

The easiest Conductor to run is built right into the [development command line tools](./intro_to_command_line_tools.md). It has no required configuration and is launched via the `hc run` command. Meant primarily for accelerating the development process it is useful for testing APIs or prototyping user interfaces.  The `hc run` command expects to be executed from inside a directory with valid DNA source files: The command is simply:
```shell
hc run
```

This will start the DNA instance in a Conductor and open, by default, a WebSocket JSON-RPC server on port `8888`. You can find more details on how to use the API in your UI in the [JSON-RPC interfaces article](./json_rpc_interfaces.md).

The following are the options for configuring `hc run`, should you need something besides the defaults.

### Packaging

`-b`/`--package`

Package your DNA before running it. Recall that to [package]() is to build the `yourapp.dna.json` file from the source files. `hc run` always looks for a DNA package file in the root of your DNA folder that should have the same name as the directory itself with suffix:  `.dna.json`, so make sure that one exists there when trying to use it. `hc run --package` will do this, or run `hc package` beforehand.

**example**
```shell
hc run --package
```

### Storage

`--persist`

Persist source chain and DHT data onto the file system. By default, none of the data being written to the source chain gets persisted beyond the running of the server. This will store data in the same directory as your DNA source code, in a hidden folder called `.hc`.

**example**
```shell
hc run --persist
```

### Interfaces

`--interface`

Select a particular JSON-RPC interface to serve your DNA instance over.

The JSON-RPC interface will expose, via a port on your device, a WebSocket or an HTTP server. It can be used to make function calls to the Zomes of a DNA instance. These are covered in depth in the [JSON-RPC interfaces article](./json_rpc_interfaces.md).

The default interface is `websocket`.

**examples**
To run it as HTTP, run:
```shell
hc run --interface http
```

To explicitly run it as WebSockets, run:
```shell
hc run --interface websocket
```

### Port

`-p`/`--port`

Customize the port number that the server runs on.

**example**
```shell
hc run --port 3400
```

### Networking

`--networked`

Select whether the Conductor should network with other nodes that are running instances of the same DNA. By default this does not occur, instead the instance runs in isolation from the network, allowing only the developer to locally access it.

This option requires more configuration, which can be read about in the
[configuring networking article](./hc_configuring_networking.md).

### Stopping the Server
Once you are done with the server, to quit type `exit` then press `Enter`, or press `Ctrl-C`.
