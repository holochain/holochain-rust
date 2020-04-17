const { one, two } = require('../config')

module.exports = scenario => {
  scenario('agentId', async (s, t) => {
    const { alice, bob } = await s.players({ alice: one, bob: one }, true)
    t.ok(alice.info('app').agentAddress)
    t.notEqual(alice.info('app').agentAddress, bob.info('app').agentAddress)
  })

  scenario('send ping', async (s, t) => {
    const { alice, bob } = await s.players({ alice: one, bob: one }, true)
    const params = { to_agent: bob.info('app').agentAddress, message: 'hello' }
    const result = await alice.call('app', 'blog', 'ping', params)
    t.deepEqual(result, { Ok: { msg_type: 'response', body: `got hello from ${alice.info('app').agentAddress}` } })
  })

  scenario.only('multiple zome calls', async (s, t) => {
    const { alice, bob } = await s.players({ alice: one, bob: one }, true)
    const params = { to_agent: bob.info('app').agentAddress, message: 'hello' }

    // shut down bob so ping to bob will timeout to complete
    await bob.kill()
    let results = []
    const f1 = alice.call('app', 'blog', 'ping', params).then(r => {results.push(2)})
    const f2 = alice.call('app',"blog", "get_test_properties", {}).then(r => {results.push(1)})
    await Promise.all([f1,f2])

    // prove that show_env returned before ping
    t.deepEqual(results,[1,2])

  })

}
