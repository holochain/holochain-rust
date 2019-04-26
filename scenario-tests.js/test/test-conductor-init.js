const test = require('tape-async')
const { Config, Conductor } = require('..')

test('can run a test', async t => {
  const someDna = Config.dna('./path/to/dna.json')
  const alice = Config.agent('alice')
  const instance1 = Config.instance(someDna, alice)

  let config = {
    dnas: [someDna],
    agents: [alice],
    instances: [instance1]
  }

  let mockHcClient = async () => ({
    call: () => async () => 'hi'
  })

  let conductor = new Conductor(config, mockHcClient)
  await conductor.connect()
  await conductor.initialize()

  t.end()
})
