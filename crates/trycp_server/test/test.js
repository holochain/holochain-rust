var WebSocket = require('rpc-websockets').Client

const { spawn } = require('child_process');
const port = "3000"
const trycp = spawn('trycp_server', ['-p', port]);
trycp.stdout.on('data', (data) => {
    var regex = new RegExp("waiting for connections on port "+port);
    if (regex.test(data)){
        console.log("boink")
        doTest()
    }
    console.log(`stdout: ${data}`);
});
trycp.stderr.on('data', (data) => {
    console.error(`stderr: ${data}`);
});


// instantiate Client and connect to an RPC server
function doTest() {
    const url = "ws://localhost:"+port
    console.log("starting up at ",url)
    var ws = new WebSocket(url)
    ws.on('open', function() {
        console.log("making ping call")
        // call an RPC method with parameters

        ws.call('ping', {"id": "my-player"}).then(function(result) {
            console.log(result)
        })

        const config_toml = "[config]"
        const config = Buffer.from(config_toml).toString('base64')
        console.log("making player call with config", config)
        ws.call('player', {"id": "my-player", "config": config}).then(function(result) {
            console.log(result)
        })

        console.log("making spawn call")
        ws.call('spawn', {"id": "my-player"}).then(function(result) {
            console.log(result)
        })

        console.log("making kill call")
        ws.call('kill', {"id": "my-player"}).then(function(result) {
            console.log(result)
            ws.close()
        })

        // close a websocket connection
        //ws.close()
    })
}
