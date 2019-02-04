# Real Networking

`hc run` uses mock networking by default and therefore doesn't talk to any other nodes.  If you wish to test multiple nodes you will need to install the [n3h](https://github.com/holochain/n3h) networking component (following the instructions on the readme).  

If you set the HC_N3H_PATH environment variable to the path where you installed it, `hc run` will automatically default to using real networking.

Set the HC_N3H_PATH environment variable, and start the server by running (make sure to change the path to where you actually installed n3h):
`HC_N3H_PATH=/home/eric/holochain/n3h hc run`

Assuming n3h was installed properly you should see something like this:
```shell
SPAWN ("node" "/home/eric/holochain/n3h/packages/n3h/bin/n3h")
(@hackmode@) [t] bound to tcp://127.0.0.1:42341
(@hackmode@) [i] p2p bound [
  "/ip4/127.0.0.1/tcp/34199/ipfs/QmTg9qMFBosfWD8yeLbcNUwT8UgwNKoT9mGEfm9vXKEHzS",
  "/ip4/192.168.1.5/tcp/34199/ipfs/QmTg9qMFBosfWD8yeLbcNUwT8UgwNKoT9mGEfm9vXKEHzS"
]
(@hackmode@) [t] running
...
```

### Starting A Second Node

Starting up a second node is a little bit more work:
1. Provide the address of the first node as a bootstrap node, by setting the `HC_N3H_BOOTSTRAP_NODE` environment variable
2. Specify a different agent id, by setting the `HC_AGENT` environment variable
3. Specify a different port than the first node to run on

For `1`, grab the string from the terminal log of the first node, the one that starts with "/ip4/192.168".

For `2`, since the first agent by default will be `testAgent`, `testAgent2` is suitable.

For `3`, since the port for the first node by default will be `8888`, `8889` is suitable.

Running the command could look like this:
``` shell
HC_AGENT=testAgent2 HC_N3H_BOOTSTRAP_NODE=/ip4/192.168.1.5/tcp/43919/ipfs/QmUhYXbBKcfL8KWx8DMpmhcHeWmmyyLHUe7jFnP5PdLdr4 HC_N3H_PATH=/home/eric/holochain/n3h hc run --port 8889
```

In the terminal logs that follow, you should see:
```shell
(libp2p) [i] QmUmUF..V71C new peer QmeDpQLchA9xeLDJ2jyXBwpe1JaQhFRrnWC2JfyyET2AAM
(libp2p) [i] QmUmUF..V71C found QmeDpQLchA9xeLDJ2jyXBwpe1JaQhFRrnWC2JfyyET2AAM in 14 ms
(libp2p) [i] QmUmUF..V71C ping round trip 37 ms
(libp2p) [i] QmUmUF..V71C got ping, sending pong
```

This means that the nodes are able to communicate! Watch the logs for gossip, as you take actions (that alter the source chain) in either node.
