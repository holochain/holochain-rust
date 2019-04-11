# Configuring Networking for `hc run`

`hc run` uses mock networking by default and therefore doesn't talk to any other nodes.

Start the server by running:
```shell
hc run
```

You should see something like this:
```shell
Network spawned with bindings:
	 - ipc: wss://127.0.0.1:64518/
	 - p2p: ["wss://192.168.0.11:64519/?a=hkYW7TrZUS1hy-i374iRu5VbZP1sSw2mLxP4TSe_YI1H2BJM3v_LgAQnpmWA_iR1W5k-8_UoA1BNjzBSUTVNDSIcz9UG0uaM"]
...
```

### Starting A Second Node

Starting up a second node is a little bit more work:

#### Step 1
Set the `HC_N3H_BOOTSTRAP_NODE` environment variable to the external p2p bound address listed by the first node. Copy-paste it from the string from the terminal log of the first node, the one that starts with "/ip4/192.168".

#### Step 2
Specify a different agent id than the first node, by setting the `HC_AGENT` environment variable. Since the first agent by default will be `testAgent`, `testAgent2` is suitable.

#### Step 3
Specify a different port than the first node to run on. Since the port for the first node by default will be `8888`, `8889` is suitable.

Running the command could look like this:
``` shell
HC_AGENT=testAgent2 HC_N3H_BOOTSTRAP_NODE=wss://192.168.0.11:64519/?a=hkYW7TrZUS1hy-i374iRu5VbZP1sSw2mLxP4TSe_YI1H2BJM3v_LgAQnpmWA_iR1W5k-8_UoA1BNjzBSUTVNDSIcz9UG0uaM hc run --port 8889
```

In the terminal logs that follow, you should see:
```shell
(libp2p) [i] QmUmUF..V71C new peer QmeDpQLchA9xeLDJ2jyXBwpe1JaQhFRrnWC2JfyyET2AAM
(libp2p) [i] QmUmUF..V71C found QmeDpQLchA9xeLDJ2jyXBwpe1JaQhFRrnWC2JfyyET2AAM in 14 ms
(libp2p) [i] QmUmUF..V71C ping round trip 37 ms
(libp2p) [i] QmUmUF..V71C got ping, sending pong
```

This means that the nodes are able to communicate! Watch the logs for gossip, as you take actions (that alter the source chain) in either node.

