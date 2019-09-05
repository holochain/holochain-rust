module.exports = scenario => {

scenario('agentId', async (s, t, { alice, bob }) => {
  t.ok(alice.app.agentId)
  t.notEqual(alice.app.agentId, bob.app.agentId)
})

scenario('show_env', async (s, t, { alice }) => {
  const result = await alice.app.call("blog", "show_env", {})

  t.equal(result.Ok.dna_address, alice.app.dnaAddress)
  t.equal(result.Ok.dna_name, "HDK-spec-rust")
  t.equal(result.Ok.agent_address, alice.app.agentId)
  t.equal(result.Ok.agent_id, '{"nick":"alice::app","pub_sign_key":"' + alice.app.agentId + '"}')
  t.equal(result.Ok.properties, '{"test_property":"test-property-value"}')

  // don't compare the public token because it changes every time we change the dna.
  t.deepEqual(result.Ok.cap_request.provenance, [ alice.app.agentId, 'HFQkrDmnSOcmGQnYNtaYZWj89rlIQVFg0PpEoeFyx/Qw6Oizy5PI+tcsO8wYrllkuVPPzF5P3pvbCctKkfyGBg==' ]
);

})

scenario('cross zome call', async (s, t, { alice }) => {

  const num1 = 2
  const num2 = 2
  const params = { num1, num2 }
  const result = await alice.app.call("blog", "check_sum", params)
  t.notOk(result.Err)
  t.equal(result.Ok, 4)
})

scenario('send ping', async (s, t, { alice, bob }) => {
  const params = { to_agent: bob.app.agentId, message: "hello" }
  const result = await alice.app.call("blog", "ping", params)
    t.deepEqual(result, { Ok: { msg_type:"response", body: "got hello from HcSCIv3cPT5kegjoqgXM7nVU8rFbd9pyg5oOYUz9PSryp5mb7DKhCsXCS768pua" } })
})

scenario('hash_memo', async (s, t, { alice }) => {

  const params = { content: "Reminder: Buy some HOT." }
  const result = await alice.app.call("blog", "memo_address", params)

  t.equal(result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")
})


  scenario('emit signal', async (s, t, { alice }) => {
    const result = await alice.app.callSync("simple", "test_emit_signal", {message: "test message"})
    t.equal(alice.app.signals.length, 1)
    t.deepEqual(alice.app.signals[0], { signal_type: 'User', name: 'test-signal', arguments: '{"message":"test message"}' })
    t.notOk(result.Err)
  })

}
