var WebSocket = require('rpc-websockets').Client


// instantiate Client and connect to an RPC server
var ws = new WebSocket('ws://localhost:9000')
console.log("starting")
ws.on('open', function() {
    console.log("making call")
    // call an RPC method with parameters
    const config_toml = "[config]"
    const config = Buffer.from(config_toml).toString('base64')
    ws.call('player', {"id": "my-player", "config": config}).then(function(result) {
        console.log("got result")
        console.log(result)
    })

    ws.call('spawn', {"id": "my-player"}).then(function(result) {
        console.log("got result")
        console.log(result)
    })

    ws.call('kill', {"id": "my-player"}).then(function(result) {
        console.log("got result")
        console.log(result)
    })

    // close a websocket connection
    ws.close()
})
