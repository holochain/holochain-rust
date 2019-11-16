import { Config } from '@holochain/tryorama'
import * as R from 'ramda'
import { ScenarioApi } from '@holochain/tryorama/lib/api'
import { Player } from '@holochain/tryorama/lib/player'
import { ConductorConfig } from '@holochain/tryorama/lib/types'
import { Batch } from '@holochain/fidget-spinner'

const dna = Config.dna('passthrough-dna.dna.json', 'passthrough')

/** Generates a bunch of identical conductor configs with multiple identical instances */
const configBatchSimple = (numConductors, numInstances) => {
  const conductor = R.pipe(
    R.map(n => [`${n}`, dna]),
    R.fromPairs,
    x => ({ instances: x }),
  )(R.range(0, numInstances))
  return R.repeat(conductor, numConductors)
}

const trace = R.tap(x => console.log('{T}', x))

module.exports = (scenario, N, M) => {

  scenario('one at a time', async (s, t) => {
    const players = R.values(await s.players(configBatchSimple(N, M), true))
    const batch = new Batch(players).iteration('series')

    // Make every instance of every conductor commit an entry

    const commitResults = await batch.mapInstances(instance =>
      instance.call('main', 'commit_entry', { content: trace(`entry-${instance.player.name}-${instance.id}`) })
    )
    const hashes = commitResults.map(x => x.Ok)

    // All results are Ok (not Err)
    t.deepEqual(commitResults.map(x => x.Err), R.repeat(undefined, N * M))
    t.ok(hashes.every(R.identity))

    await s.consistency()

    // Make one instance commit an entry as a base and link every previously committed entry as a target

    const instance1 = await players[0]._instances[0]
    const baseHash = await instance1.call('main', 'commit_entry', { content: 'base' }).then(r => r.Ok)
    let addLinkResults = []
    for (const hash of hashes) {
      const result = await instance1.call('main', 'link_entries', { base: baseHash, target: hash })
      addLinkResults.push(result)
    }

    await s.consistency()

    t.ok(addLinkResults.every(r => r.Ok))
    t.equal(addLinkResults.length, N * M)
    t.deepEqual(addLinkResults.map(x => x.Err), R.repeat(undefined, N * M))

    // Make each other instance getLinks on the base hash

    const getLinksResults = await batch.mapInstances(instance => instance.call('main', 'get_links', { base: baseHash }))

    // All getLinks results contain the full set
    t.deepEqual(getLinksResults.map(r => r.Ok.links.length), R.repeat(N * M, N * M))
  })

  scenario('all at once', async (s, t) => {
    const players = R.values(await s.players(configBatchSimple(N, M), true))
    const batch = new Batch(players).iteration('parallel')

    const commitResults = await batch.mapInstances(instance =>
      instance.call('main', 'commit_entry', { content: trace(`entry-${instance.player.name}-${instance.id}`) })
    )
    const hashes = commitResults.map(x => x.Ok)

    // All results are Ok (not Err)
    t.deepEqual(commitResults.map(x => x.Err), R.repeat(undefined, N * M))
    t.ok(hashes.every(R.identity))

    await s.consistency()

    const instance1 = await players[0]._instances['0']
    const baseHash = await instance1.call('main', 'commit_entry', { content: 'base' }).then(r => r.Ok)
    const addLinkResults: Array<any> = await Promise.all(
      hashes.map(hash => instance1.call('main', 'link_entries', { base: baseHash, target: hash }))
    )

    t.ok(addLinkResults.every(r => r.Ok))
    t.equal(addLinkResults.length, N * M)
    t.deepEqual(addLinkResults.map(x => x.Err), R.repeat(undefined, N * M))

    await s.consistency()

    const getLinksResults = await batch.mapInstances(instance => instance.call('main', 'get_links', { base: baseHash }))

    // All getLinks results contain the full set
    t.deepEqual(getLinksResults.map(r => r.Ok.links.length), R.repeat(N * M, N * M))
  })
}
