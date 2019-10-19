var WebSocket = require('rpc-websockets').Client


// instantiate Client and connect to an RPC server
var ws = new WebSocket('ws://localhost:9000')
console.log("starting")
ws.on('open', function() {
    console.log("making ping call")
    // call an RPC method with parameters

    ws.call('ping', {"id": "my-player"}).then(function(result) {
        console.log(result)
    })

    const config_toml = "[config]"
    const config = Buffer.from(config_toml).toString('base64')
    console.log("making player call")
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
