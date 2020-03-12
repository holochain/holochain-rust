const { oneOffline } = require('./config')

module.exports = scenario => {
    scenario('OFFLINE: alice create & publish post -> recommend own post to self', async (s, t) => {
        const { alice } = await s.players({ alice: oneOffline }, true)

        const content = 'Holo world...1'
        const params = { content: content, in_reply_to: null }
        const postResult = await alice.callSync('app', 'blog', 'create_post', params)
        const postAddr = postResult.Ok
        t.ok(postAddr, `error: ${postResult}`)

        const gotPost = await alice.call('app', 'blog', 'get_post', { post_address: postAddr })
        t.ok(gotPost.Ok)

        const linked = await alice.callSync('app', 'blog', 'recommend_post', {
            post_address: postAddr,
            agent_address: alice.info('app').agentAddress
        })
        console.log('linked: ', linked)
        t.ok(linked.Ok)

        const recommendedPosts = await alice.call('app', 'blog', 'my_recommended_posts', {})
        console.log('recommendedPosts', recommendedPosts)
        console.log('agent addresses: ', alice.info('app').agentAddress, alice.info('app').agentAddress)
        t.equal(recommendedPosts.Ok.links.length, 1)
    })
}