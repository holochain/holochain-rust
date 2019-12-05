import * as R from 'ramda'
import { Batch } from '@holochain/tryorama-stress-utils'
import {configBatchSimple} from './config'

const trace = R.tap(x => console.log('{T}', x))
const delay = ms => new Promise(r => setTimeout(r, ms))

module.exports = (scenario, N, M) => {

  scenario('all instances hold the same aspects after startup (smooth)', async (s, t) => {
    const players = R.values(await s.players(configBatchSimple(N, M), true))

    // TODO: we shouldn't even need this delay
    await delay(10000)
    
    const batch = new Batch(players).iteration('series')

    const holds = await batch.mapInstances(async instance => {
      const dump = await instance.stateDump()
      const hashes = Object.keys(dump.held_aspects)
      return hashes.length
    })

    // We expect to hold 6 entries for each instance in the network:
    // - DNA
    // - AgentId
    // - CapToken
    // and headers for each of these
    const expected = (N * M) * 6
    t.deepEqual(holds, R.repeat(expected, N * M))
  })

}
