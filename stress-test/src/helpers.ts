import * as R from 'ramda'

import { Player } from '@holochain/tryorama/lib/player'
import { Instance } from '@holochain/tryorama/lib/instance'

/**
 * Takes an object whose keys correspond to array indices,
 * and construct a true array
 */
const indexedObjectToArray = <T>(o: { [n: string]: T }): Array<T> => {
  const r = (arr, pair) => {
    arr[pair[0]] = pair[1]
    return arr
  }

  return R.pipe(
    R.toPairs,
    R.reduce(r, []),
  )(o)
}

class Batch {
  members: Array<[Player, Array<Instance>]>

  constructor(members: Array<Player>) {
    this.members = members.map(p => [p, indexedObjectToArray(p._instances)])

  }

  call = (zome, func, params) => {
    this.members.map(([player, instances], p) =>
      instances.map((instance, i) => {
        return instance.call('main', 'commit_entry', { content: `entry-${p}-${i}` })
      })
    )
  }
}


export const everybody = async (players: Array<Player>) => {
  await R.pipe(
    // Flatten into a 1d array
    R.flatten,
    // Await all in parallel
    x => Promise.all(x),
  )(
    players.map((player, p) =>
      R.values(player._instances).map((instance, i) => {
        return instance.call('main', 'commit_entry', { content: `entry-${p}-${i}` })
      })
    )
  )
}
