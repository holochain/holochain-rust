# Running An App

For the purpose of *testing* APIs or prototyping user interfaces, you can run a DNA from the directory it's contained. The most basic way to do this is to run:
```shell
hc run
```
This will start the application and open, by default, a WebSocket JSON-RPC server on port `8888`. This is covered in more depth in the [JSON-RPC interfaces article](./json_rpc_interfaces.md).

There are three option flags for `hc run`.

If you wish to customize the port number that the server runs on, then run it with a `-p`/`--port` option, like:
```shell
hc run --port 3400
```

If you wish to "package" your DNA before running it, which is to build the `bundle.json` file from the source files, then use the `-b`/`--package` option, like:
```shell
hc run --package
```
Note that `hc run` always looks for a `bundle.json` file in the root of your app folder, so make sure that one exists there when trying to use it. `hc run --package` will do this, or run `hc package`.

By default, none of the data your application is writing to the source chain gets persisted. If you wish to persist data onto the file system, use the `--persist` flag, like:
```shell
hc run --persist
```
This will store data in the same directory as your app, in a hidden folder called `.hc`.

Of course these options can be used in combination with one another.

Once you are done with the server, to quit just type `exit` then press `Enter`, or just press `Ctrl-C`.

There are two further configuration options which are deserving of their own articles, which follow this one: [selecting and using the different JSON-RPC interfaces](./json_rpc_interfaces.md), and [using real networking](./real_networking.md).