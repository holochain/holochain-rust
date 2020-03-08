# Configuring Networking for `hc run`

`hc run` uses mock networking by default and therefore doesn't talk to any other nodes.

In order to have `hc run` spawn a real network instance, start it with the `--networked` option:
```shell
hc run --networked
```
