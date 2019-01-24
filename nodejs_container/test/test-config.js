const test = require('tape')

const { Config } = require('..')
const C = Config

test('agent construction', t => {
    const name = 'alice'
    const agent = C.agent(name)
    t.deepEqual(agent, { name })
    t.end()
})

test('DNA construction with implicit name', t => {
    const path = 'path/to/dna'
    const dna = C.dna(path)
    t.deepEqual(dna, { path, name: path })
    t.end()
})

test('DNA construction with explicit name', t => {
    const path = 'path/to/dna'
    const dna = C.dna(path, 'george')
    t.deepEqual(dna, { path, name: 'george' })
    t.end()
})

test('instance construction with implicit name', t => {
    const path = 'path/to/dna'
    const agent = C.agent('allison')
    const dna = C.dna(path, 'dnaName')
    const instance = C.instance(agent, dna)
    t.deepEqual(instance, { agent, dna, name: 'allison' })
    t.end()
})

test('instance construction with explicit name', t => {
    const path = 'path/to/dna'
    const agent = C.agent('konstantin')
    const dna = C.dna(path)
    const instance = C.instance(agent, dna, 'kostya')
    t.deepEqual(instance, { agent, dna, name: 'kostya' })
    t.end()
})

test('config construction', t => {
    const path = 'path/to/dna'
    const agent1 = C.agent('alessia')
    const agent2 = C.agent('bartolini')
    const dna = C.dna(path)
    const config = C.container([
        C.instance(agent1, dna),
        C.instance(agent2, dna),
    ])
    t.deepEqual(
        config.agents.map(a => a.id).sort(),
        ['alessia', 'bartolini']
    )
    t.deepEqual(
        config.dnas.map(d => d.id).sort(),
        [path]
    )
    t.equal(config.instances[0].id, `alessia`)
    t.equal(config.instances[0].agent, `alessia`)
    t.equal(config.instances[0].dna, path)
    t.equal(config.instances[1].id, `bartolini`)
    t.equal(config.instances[1].agent, `bartolini`)
    t.equal(config.instances[1].dna, path)
    t.equal(config.interfaces.length, 0)
    t.equal(config.bridges.length, 0)
    t.end()
})

