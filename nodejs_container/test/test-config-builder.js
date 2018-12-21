const test = require('tape')

const { ConfigBuilder } = require('..')
const H = ConfigBuilder

test('agent construction', t => {
    const name = 'alice'
    const agent = H.agent(name)
    t.deepEqual(agent, { name })
    t.end()
})

test('DNA construction', t => {
    const path = 'path/to/dna'
    const dna = H.dna(path)
    t.deepEqual(dna, { path })
    t.end()
})

test('instance construction', t => {
    const path = 'path/to/dna'
    const agent = H.agent('allison')
    const dna = H.dna(path)
    const instance = H.instance(agent, dna)
    t.deepEqual(instance, { agent, dna })
    t.end()
})

test('config construction', t => {
    const path = 'path/to/dna'
    const agent1 = H.agent('alessia')
    const agent2 = H.agent('bartolini')
    const dna = H.dna(path)
    const config = H.container(
        H.instance(agent1, dna),
        H.instance(agent2, dna),
    )
    t.deepEqual(
        config.agents.map(a => a.id).sort(),
        ['alessia', 'bartolini']
    )
    t.deepEqual(
        config.dnas.map(d => d.id).sort(),
        [path]
    )
    t.equal(config.instances[0].id, `alessia-${path}`)
    t.equal(config.instances[0].agent, `alessia`)
    t.equal(config.instances[0].dna, path)
    t.equal(config.instances[1].id, `bartolini-${path}`)
    t.equal(config.instances[1].agent, `bartolini`)
    t.equal(config.instances[1].dna, path)
    t.equal(config.interfaces.length, 0)
    t.equal(config.bridges.length, 0)
    t.end()
})

