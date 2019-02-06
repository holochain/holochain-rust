const path = require('path')
const { Config, Conductor, Scenario } = require('../../nodejs_conductor')
Scenario.setTape(require('tape'))

const dnaPath = path.join(__dirname, "../dist/app_spec.hcpkg")
const dna = Config.dna(dnaPath, 'app-spec')
const agentAlice = Config.agent("alice")
const agentTash = Config.agent("tash")

const instanceAlice = Config.instance(agentAlice, dna)
const instanceTash = Config.instance(agentTash, dna)

const scenario = new Scenario([instanceAlice, instanceTash])

scenario.runTape('calling get_links before link_entries makes no difference', async (t, {alice}) => {

  const get1 = alice.call("blog", "my_posts", {})
  t.ok(get1.Ok)

  const create1 = await alice.callSync("blog","create_post", {content: 'hi'})
  t.ok(create1.Ok)

  const get2 = alice.call("blog", "my_posts", {})
  t.ok(get2.Ok)

  t.equal(get2.Ok.addresses.length, 1)
})

scenario.runTape('calling get_links twice in a row is no different than calling it once', async (t, {alice}) => {
  // This test is exactly the same as the previous one, but calls my_posts twice in a row.
  // This makes the links come through the second time.

  const get1 = alice.call("blog", "my_posts", {})
  t.ok(get1.Ok)

  const create1 = await alice.callSync("blog", "create_post", {content: 'hi'})
  t.ok(create1.Ok)

  alice.call("blog", "my_posts", {})
  const get2 = alice.call("blog", "my_posts", {})
  t.ok(get2.Ok)

  t.equal(get2.Ok.addresses.length, 1)
})

scenario.runTape('not calling get_links in the beginning is also ok', async (t, {alice}) => {

  const create1 = await alice.callSync("blog", "create_post", {content: 'hi'})
  t.ok(create1.Ok)

  const get1 = alice.call("blog", "my_posts", {})
  t.ok(get1.Ok)

  t.equal(get1.Ok.addresses.length, 1)
})

scenario.runTape('alice create & publish post -> recommend own post to self', async (t, {alice, tash}) => {

  const content = "Holo world...1"
  const params = { content: content, in_reply_to: null }
  const postResult = await alice.callSync("blog", "create_post", params)
  const postAddr = postResult.Ok
  t.ok(postAddr, `error: ${postResult}`)

  const gotPost = alice.call("blog", "get_post", {post_address: postAddr})
  t.ok(gotPost.Ok)

  let linked = await alice.callSync('blog', 'recommend_post', {
    post_address: postAddr,
    agent_address: alice.agentId
  })
  console.log("linked: ", linked)
  t.equal(linked.Ok, null)

  const recommendedPosts = alice.call('blog', 'my_recommended_posts', {})
  console.log("recommendedPosts", recommendedPosts)
  console.log('agent addresses: ', alice.agentId, alice.agentId)

  t.equal(recommendedPosts.Ok.addresses.length, 1)
})

scenario.runTape('alice create & publish post -> tash recommend to self', async (t, {alice, tash}) => {
  const content = "Holo world...2"
  const params = { content: content, in_reply_to: null }
  const postResult = await alice.callSync("blog", "create_post", params)
  const postAddr = postResult.Ok
  t.ok(postAddr, `error: ${postResult}`)

  const gotPost = tash.call("blog", "get_post", {post_address: postAddr})
  t.ok(gotPost.Ok)

  let linked = await tash.callSync('blog', 'recommend_post', {
    post_address: postAddr,
    agent_address: tash.agentId
  })
  console.log("linked: ", linked)
  t.equal(linked.Ok, null)

  const recommendedPosts = tash.call("blog", "my_recommended_posts", {})
  console.log("recommendedPosts", recommendedPosts)
  console.log('agent addresses: ', alice.agentId, tash.agentId)

  t.equal(recommendedPosts.Ok.addresses.length, 1)
})

scenario.runTape('create & publish post -> recommend to other agent', async (t, {alice, tash}) => {
  const content = "Holo world...3"
  const params = { content: content, in_reply_to: null }
  const postResult = await alice.callSync("blog", "create_post", params)
  const postAddr = postResult.Ok
  t.ok(postAddr, `error: ${postResult}`)

  const gotPost = tash.call("blog", "get_post", {post_address: postAddr})
  t.ok(gotPost.Ok)

  let linked = await alice.callSync('blog', 'recommend_post', {
    post_address: postAddr,
    agent_address: tash.agentId
  })
  console.log("linked: ", linked)
  t.equal(linked.Ok, null)

  const recommendedPosts = tash.call('blog', 'my_recommended_posts', {})
  console.log("recommendedPosts", recommendedPosts)
  console.log('agent addresses: ', alice.agentId, tash.agentId)

  t.equal(recommendedPosts.Ok.addresses.length, 1)
})
