const { one } = require('../config')

module.exports = scenario => {
    scenario.only('get zomes by trait', async (s, t) => {
        const {conductor} = await s.players({conductor: one}, true)

        const crypto_trait = {
            name: 'crypto',
            functions: [
                {
                    name: 'encrypt',
                    inputs: { payload: 'String' },
                    outputs: { result: 'String' },
                },
                {
                    name: 'decrypt',
                    inputs: { payload: 'String' },
                    outputs: { result: 'String' },
                }
            ]
        };

        const zomes = conductor.admin('introspection/traits/get_zomes_by_trait', crypto_trait)

        t.deepEqual(zomes, ['app/simple'])
    })
}