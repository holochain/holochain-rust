const { one, two } = require('../config')

module.exports = scenario => {

scenario('agentId', async (s, t) => {
      const { alice, bob } = await s.players({alice: one, bob: one}, true)
  t.ok(alice.info('app').agentAddress)
  t.notEqual(alice.info('app').agentAddress, bob.info('app').agentAddress)
})


scenario('send ping', async (s, t) => {
      const { alice, bob } = await s.players({alice: one, bob: one}, true)
  const params = { to_agent: bob.info('app').agentAddress, message: "hello" }
  const result = await alice.call('app', "blog", "ping", params)
    t.deepEqual(result, { Ok: { msg_type:"response", body: `got hello from ${alice.info('app').agentAddress}` } })
})


}
