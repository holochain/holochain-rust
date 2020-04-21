const { one } = require('../config')

module.exports = scenario => {
    scenario.only('get zomes by trait', async (s, t) => {
        const {conductor} = await s.players({conductor: one}, true)

        const crypto_trait = {
            name: 'crypto',
            functions: [
                {
                    name: 'encrypt',
                    inputs: [{ name: 'payload', type: 'String' }],
                    outputs: [{ name: 'result', type: 'String' }],
                },
                {
                    name: 'decrypt',
                    inputs: [{ name: 'payload', type: 'String' }],
                    outputs: [{ name: 'result', type: 'String' }],
                }
            ]
        };

        const zomes = await conductor.admin('introspection/traits/get_zomes_by_trait', {trait: crypto_trait})

        t.deepEqual(zomes, ['app/simple'])
    })
}