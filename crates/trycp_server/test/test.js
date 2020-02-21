var WebSocket = require('rpc-websockets').Client


process.on('unhandledRejection', error => {
    console.error('got unhandledRejection:', error);
});

//doTest("ws://localhost:9000")
//doTest("wss://test1-eu-central-1.holochain-aws.org")
//magic_remote_machine_manager("3000")
function magic_remote_machine_manager(port) {
    const { spawn } = require('child_process');
    const trycp = spawn('trycp_server', ['-p', port]);
    trycp.stdout.on('data', (data) => {
        var regex = new RegExp("waiting for connections on port "+port);
        if (regex.test(data)){
            doTest("ws://localhost:"+port)
        }
        console.log(`stdout: ${data}`);
    });
    trycp.stderr.on('data', (data) => {
        console.error(`stderr: ${data}`);
    });
}

// instantiate Client and connect to an RPC server
function  doTest(url) {
    return new Promise( (resolve) => {
    console.log("starting up at ",url)
    var ws = new WebSocket(url)
    ws.on('open', async function() {
        console.log("making ping call")
        // call an RPC method with parameters

        await ws.call('ping', {"id": "my-player"}).then(function(result) {
             result = JSON.parse(result)
             console.log(result)
        })

        console.log("making setup call")
        // call an RPC method with parameters

        await ws.call('setup', {"id": "my-player"}).then(function(result) {
            console.log(result)
        })

        await ws.call('dna', {"url": "https://github.com/holochain/passthrough-dna/releases/download/v0.0.6/passthrough-dna.dna.json"}).then(function(result) {
            console.log(result)
        })

        // call again to test caching
        await ws.call('dna', {"url": "https://github.com/holochain/passthrough-dna/releases/download/v0.0.6/passthrough-dna.dna.json"}).then(function(result) {
            console.log(result)
        })

        const config_toml =`
persistence_dir = "/tmp/somepath"

agents = []
dnas = []
instances = []

[signals]
consistency = false
trace = true

[[interfaces]]
admin = true
id = "someadminid"
instances = []
    [interfaces.driver]
    type = "websocket"
    port = 1112

[[interfaces]]
admin = true
id = "somednaid"
instances = []
    [interfaces.driver]
    type = "websocket"
    port = 1111

[logger]
type = "debug"

[network]
type = "sim2h"
sim2h_url = "wss://localhost:9001"
    `

        const config = Buffer.from(config_toml).toString('base64')
        console.log("making player call with config", config)
        let result = await ws.call('player', {"id": "my-player", "config": config})
        console.log(result)

        console.log("making player call with config", config)
        result = await ws.call('player', {"id": "my-player2", "config": config})
        console.log(result)

        console.log("making spawn call")
        result = await ws.call('spawn', {"id": "my-player"})
        console.log(result)

/*        console.log("making kill call")
        result = await ws.call('kill', {"id": "my-player"})
        console.log(result)

        console.log("making spawn call2")
        result = await ws.call('spawn', {"id": "my-player"})
        console.log(result)
*/
        console.log("making reset call")
        result = await ws.call('reset', {})
        console.log(result)

        console.log("making player2 call with config", config)
        result = await ws.call('player', {"id": "my-player", "config": config})
        console.log(result)

        // close a websocket connection
        ws.close()

        resolve()
    })
    })
}

doTestManager("ws://localhost:9000")
// instantiate Client and connect to an RPC server
function  doTestManager(url) {
    return new Promise( (resolve) => {
    console.log("starting up at ",url)
    var ws = new WebSocket(url)
    ws.on('open', async function() {
        console.log("making register call, expect: 'registered'")
        // call an RPC method with parameters
        await ws.call('register', {"url": "ws://localhost:9001"}).then(function(result) {
          console.log(result)
        })

      console.log("making request call, expect: insufficient endpoints available")
      // call an RPC method with parameters
      await ws.call('request', {"count": 100}).then(function(result) {
        console.log(result.error)
      })

      console.log("making request call, expect: registered node")
      // call an RPC method with parameters
      await ws.call('request', {"count": 1}).then(function(result) {
        console.log(result)
      })

      console.log("making request call, expect: insufficient endpoints available")
      // call an RPC method with parameters
      await ws.call('request', {"count": 1}).then(function(result) {
        console.log(result.error)
      })

        // close a websocket connection
        ws.close()

        resolve()
    })
    })
}
