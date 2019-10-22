var WebSocket = require('rpc-websockets').Client


//doTest("ws://localhost:9000")
magic_remote_machine_manager("3000")
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
async function  doTest(url) {
    console.log("starting up at ",url)
    var ws = new WebSocket(url)
    ws.on('open', async function() {
        console.log("making ping call")
        // call an RPC method with parameters

        ws.call('ping', {"id": "my-player"}).then(function(result) {
            console.log(result)
        })

        const config_toml = "[config]"  // THIS IS A BROKEN CONFIG
        const config = Buffer.from(config_toml).toString('base64')
        console.log("making player call with config", config)
        let result = await ws.call('player', {"id": "my-player", "config": config})
        console.log(result)

        console.log("making spawn call")
        result = await ws.call('spawn', {"id": "my-player"})
        console.log(result)

        console.log("making kill call")
        result = await ws.call('kill', {"id": "my-player"})
        console.log(result)

        // close a websocket connection
        ws.close()
    })
}
