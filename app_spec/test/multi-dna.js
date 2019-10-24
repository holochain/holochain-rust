const { one, two } = require('./config')

module.exports = scenario => {
  scenario('scenario test create & publish -> getting post via bridge (multi dna)', async (s, t) => {
    const { conductor } = await s.players({ conductor: two }, true)
    const initialContent = 'Holo world'
    const params = { content: initialContent, in_reply_to: null }
    const create_result = await conductor.callSync('app2', 'blog', 'create_post', params)

    t.equal(create_result.Ok, 'QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk')
    t.notEqual(conductor.info('app1').dnaAddress, conductor.info('app2').dnaAddress)

    const post_address = create_result.Ok
    const params_get = { post_address }

    const result = await conductor.call('app1', 'blog', 'get_post_bridged', params_get)
    console.log('BRIDGE CALL RESULT: ' + JSON.stringify(result))
    const value = JSON.parse(result.Ok.App[1])
    t.equal(value.content, initialContent)
  })
}
