var WebSocket = require('rpc-websockets').Client

/*process.on('unhandledRejection', error => {
  console.error('got unhandledRejection:', error);
});*/

call(process.argv[2],process.argv[3],process.argv[4])
// instantiate Client and connect to an RPC server
function  call(url, method, params) {
  const p = JSON.parse(params)
  return new Promise( (resolve) => {
    var ws = new WebSocket(url)
    ws.on('open', async function() {
      // call an RPC method with parameters
      await ws.call(method, p).then(function(result) {
        console.log(result)
      })
      // close a websocket connection
      ws.close()

      resolve()
    })
  })
}
