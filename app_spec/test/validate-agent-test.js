const { one, two } = require('./config')

module.exports = scenario => {
  scenario('An agent that does not pass validate_agent will not have a visible entry in the DHT', async (s, t) => {
    const { valid_agent, reject_agent } = await s.players({ valid_agent: one, reject_agent: one }, true)
    const get_self_result = await valid_agent.call('app', 'simple', 'get_entry', { address: valid_agent.info('app').agentAddress })
    const get_other_result = await valid_agent.call('app', 'simple', 'get_entry', { address: reject_agent.info('app').agentAddress })
    console.log('get self response', get_self_result)
    console.log('get invalid response', get_self_result)
    t.ok(get_self_result.Ok, 'Should be able to retrieve own agent entry')
    t.notOk(get_other_result.Ok, 'Should not be able to retrieve agent entry for invalid agent')
  })
}
