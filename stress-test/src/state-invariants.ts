import * as R from 'ramda'
import { Batch } from '@holochain/tryorama-stress-utils'
import {configBatchSimple} from './config'

const trace = R.tap(x => console.log('{T}', x))
const delay = ms => new Promise(r => setTimeout(r, ms))

module.exports = (scenario, N, M) => {

  scenario('all instances hold the same aspects after a smooth startup', async (s, t) => {
    const players = R.values(await s.players(configBatchSimple(N, M), true))
    const batch = new Batch(players).iteration('series')

    // Ensure that everyone holds the same number of nonzero entries
    // This is mainly a test of the consistency model
    const holds1 = await batch.mapInstances(getHoldCount)
    t.ok(holds1[0] !== 0)
    t.deepEqual(holds1, R.repeat(holds1[0], N * M))

    await delay(10000)

    // Again, ensure that everyone holds the same number of nonzero entries
    // This is the easiest test to pass, more of a sanity check
    const holds2 = await batch.mapInstances(getHoldCount)
    t.ok(holds2[0] !== 0)
    t.deepEqual(holds2, R.repeat(holds2[0], N * M))

    // More specifically, we expect to hold 6 entries for each instance in the network:
    // - DNA
    // - AgentId
    // - CapToken
    // and headers for each of these
    const expected = (N * M) * 6
    t.deepEqual(holds2, R.repeat(expected, N * M))
  })

}

const dumpGetter = getter => async instance => {
  return getter(await instance.stateDump())
}

const getHoldCount = dumpGetter(dump => Object.keys(dump.held_aspects).length)
