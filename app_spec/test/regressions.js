module.exports = scenario => {

scenario('calling get_links before link_entries makes no difference', async (s, t, {alice}) => {

  const get1 = await alice.app.call("blog", "my_posts", {})
  t.ok(get1.Ok)

  const create1 = await alice.app.callSync("blog","create_post", {content: 'hi'})
  t.ok(create1.Ok)

  const get2 = await alice.app.call("blog", "my_posts", {})
  t.ok(get2.Ok)
  t.equal(get2.Ok.links.length,1)
})

scenario('calling get_links twice in a row is no different than calling it once', async (s, t, {alice}) => {
  // This test is exactly the same as the previous one, but calls my_posts twice in a row.
  // This makes the links come through the second time.

  const get1 = await alice.app.call("blog", "my_posts", {})
  t.ok(get1.Ok)

  const create1 = await alice.app.callSync("blog", "create_post", {content: 'hi'})
  t.ok(create1.Ok)

  await alice.app.call("blog", "my_posts", {})
  const get2 = await alice.app.call("blog", "my_posts", {})
  t.ok(get2.Ok)
  t.equal(get2.Ok.links.length, 1)
})

scenario('not calling get_links in the beginning is also ok', async (s, t, {alice}) => {

  const create1 = await alice.app.callSync("blog", "create_post", {content: 'hi'})
  t.ok(create1.Ok)

  const get1 = await alice.app.call("blog", "my_posts", {})
  t.ok(get1.Ok)
  t.equal(get1.Ok.links.length, 1)
})

scenario('alice create & publish post -> recommend own post to self', async (s, t, {alice, bob}) => {

  const content = "Holo world...1"
  const params = { content: content, in_reply_to: null }
  const postResult = await alice.app.callSync("blog", "create_post", params)
  const postAddr = postResult.Ok
  t.ok(postAddr, `error: ${postResult}`)

  const gotPost = await alice.app.call("blog", "get_post", {post_address: postAddr})
  t.ok(gotPost.Ok)

  let linked = await alice.app.callSync('blog', 'recommend_post', {
    post_address: postAddr,
    agent_address: alice.app.agentId
  })
  console.log("linked: ", linked)
  t.ok(linked.Ok);

  const recommendedPosts = await alice.app.call('blog', 'my_recommended_posts', {})
  console.log("recommendedPosts", recommendedPosts)
  console.log('agent addresses: ', alice.app.agentId, alice.app.agentId)
  t.equal(recommendedPosts.Ok.links.length, 1)
})

scenario('alice create & publish post -> bob recommend to self', async (s, t, {alice, bob}) => {
  const content = "Holo world...2"
  const params = { content: content, in_reply_to: null }
  const postResult = await alice.app.callSync("blog", "create_post", params)
  const postAddr = postResult.Ok
  t.ok(postAddr, `error: ${postResult}`)

  const gotPost = await bob.app.call("blog", "get_post", {post_address: postAddr})
  t.ok(gotPost.Ok)

  let linked = await bob.app.callSync('blog', 'recommend_post', {
    post_address: postAddr,
    agent_address: bob.app.agentId
  })
  console.log("linked: ", linked)
  t.ok(linked.Ok);

  const recommendedPosts = await bob.app.call("blog", "my_recommended_posts", {})
  console.log("recommendedPosts", recommendedPosts)
  console.log('agent addresses: ', alice.app.agentId, bob.app.agentId)
  t.equal(recommendedPosts.Ok.links.length, 1)
})

scenario('create & publish post -> recommend to other agent', async (s, t, {alice, bob}) => {
  const content = "Holo world...3"
  const params = { content: content, in_reply_to: null }
  const postResult = await alice.app.callSync("blog", "create_post", params)
  const postAddr = postResult.Ok
  t.ok(postAddr, `error: ${postResult}`)

  const gotPost = await bob.app.call("blog", "get_post", {post_address: postAddr})
  t.ok(gotPost.Ok)

  let linked = await alice.app.callSync('blog', 'recommend_post', {
    post_address: postAddr,
    agent_address: bob.app.agentId
  })
  console.log("linked: ", linked)
  t.ok(linked.Ok);

  const recommendedPosts = await bob.app.call('blog', 'my_recommended_posts', {})
  console.log("recommendedPosts", recommendedPosts)
  console.log('agent addresses: ', alice.app.agentId, bob.app.agentId)
  t.equal(recommendedPosts.Ok.links.length, 1)
})

}
