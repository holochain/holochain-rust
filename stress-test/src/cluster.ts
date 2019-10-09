
// class Cluster {
//   players: Array<Player>

//   constructor(players: Array<Player>) {
//     this.players = players
//   }

//   choose = (pred: ((any, number?) => boolean)): Cluster => new Cluster(this.players.filter(pred))
//   all = () => this

//   run = (f: (Player) => Promise<any>): Promise<Array<any>> => Promise.all(this.players.map(f))
// }

// const even = (x, i) => i % 2 === 0
// const odd = (x, i) => i % 2 === 1