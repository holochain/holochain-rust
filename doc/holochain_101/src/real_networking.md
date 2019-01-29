# Real Networking

`hc run` uses mock networking by default and therefore can't talk to any other nodes.  If you want to test multiple nodes you will need to install the [n3h](https://github.com/holochain/n3h) networking component (following the instructions on the readme there).  Once you have installed it then you can simply fire up your first node while setting the HC_N3H_PATH environment variable to the path where you installed it.  If n3h was installed properly you should see something like this:

``` shell
$ HC_N3H_PATH=/home/eric/holochain/n3h hc run
SPAWN ("node" "/home/eric/holochain/n3h/packages/n3h/bin/n3h")
(@hackmode@) [t] bound to tcp://127.0.0.1:42341
(@hackmode@) [i] p2p bound [
  "/ip4/127.0.0.1/tcp/34199/ipfs/QmTg9qMFBosfWD8yeLbcNUwT8UgwNKoT9mGEfm9vXKEHzS",
  "/ip4/192.168.1.5/tcp/34199/ipfs/QmTg9qMFBosfWD8yeLbcNUwT8UgwNKoT9mGEfm9vXKEHzS"
]
(@hackmode@) [t] running
...
```
Note that there is an agent id set by default, and the default is `testAgent`.
To fire up a second node you have to do a little more work, namely:
1. providing the address of the first node as a bootstrap node,
2. specifying a different agent id
3. specifying a different port for the websocket server, for a UI to connect to.

Do that something like this (where the node address is copied from the output of the first node):

``` shell
HC_AGENT=testAgent2 HC_N3H_BOOTSTRAP_NODE=/ip4/192.168.1.5/tcp/43919/ipfs/QmUhYXbBKcfL8KWx8DMpmhcHeWmmyyLHUe7jFnP5PdLdr4 HC_N3H_PATH=/home/eric/holochain/n3h hc run -p 8889

```

In both cases make sure to change the path to where you actually installed n3h.
