module.exports = scenario => {

scenario('agentId', async (s, t, { alice, bob }) => {
  t.ok(alice.app.agentId)
  t.notEqual(alice.app.agentId, bob.app.agentId)
})


scenario('send ping', async (s, t, { alice, bob }) => {
  const params = { to_agent: bob.app.agentId, message: "hello" }
  const result = await alice.app.call("blog", "ping", params)
    t.deepEqual(result, { Ok: { msg_type:"response", body: "got hello from HcSCIv3cPT5kegjoqgXM7nVU8rFbd9pyg5oOYUz9PSryp5mb7DKhCsXCS768pua" } })
})


  scenario('emit signal', async (s, t, { alice }) => {
    const result = await alice.app.callSync("simple", "test_emit_signal", {message: "test message"})
    t.equal(alice.app.signals.length, 1)
    t.deepEqual(alice.app.signals[0], { signal_type: 'User', name: 'test-signal', arguments: '{"message":"test message"}' })
    t.notOk(result.Err)
  })

}
