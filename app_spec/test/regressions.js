const path = require('path')
const { Config, Container, Scenario } = require('../../nodejs_container')
Scenario.setTape(require('tape'))

const dnaPath = path.join(__dirname, "../dist/app_spec.hcpkg")
const dna = Config.dna(dnaPath, 'app-spec')
const agentAlice = Config.agent("alice")
const agentTash = Config.agent("tash")

const scenario = new Scenario([instanceAlice, instanceTash])

scenario.runTape('calling get_links before link_entries makes a difference', async (t, {alice}) => {

  const get1 = alice.call("blog", "main", "my_posts", {})
  t.ok(get1.Ok)
  sleep.sleep(1)

  const create1 = await alice.callSync("blog", "main", "create_post", {content: 'hi'})
  t.ok(create1.Ok)
  sleep.sleep(1)

  const get2 = alice.call("blog", "main", "my_posts", {})
  t.ok(get2.Ok)

  t.equal(get2.Ok.addresses.length, 1)
  t.end()
})

scenario.runTape('calling get_links twice in a row is different than calling it once', async (t, {alice}) => {
  // This test is exactly the same as the previous one, but calls my_posts twice in a row.
  // This makes the links come through the second time.

  const get1 = alice.call("blog", "main", "my_posts", {})
  t.ok(get1.Ok)
  sleep.sleep(1)

  const create1 = await alice.callSync("blog", "main", "create_post", {content: 'hi'})
  t.ok(create1.Ok)
  sleep.sleep(1)

  alice.call("blog", "main", "my_posts", {})
  const get2 = alice.call("blog", "main", "my_posts", {})
  t.ok(get2.Ok)

  t.equal(get2.Ok.addresses.length, 1)
  t.end()
})

scenario.runTape('not calling get_links in the beginning helps', async (t, {alice}) => {

  const create1 = await alice.callSync("blog", "main", "create_post", {content: 'hi'})
  t.ok(create1.Ok)
  sleep.sleep(1)

  const get1 = alice.call("blog", "main", "my_posts", {})
  t.ok(get1.Ok)

  t.equal(get1.Ok.addresses.length, 1)
  t.end()
})

scenario.runTape('alice create & publish post -> recommend own post to self', async (t, {alice, tash}) => {

  const params = { content: content, in_reply_to }
  const postResult = await alice.callSync("blog", "main", "create_post", params)
  const postAddr = postResult.Ok
  t.ok(postAddr, `error: ${postResult}`)

  const gotPost = alice.call("blog", "main", "get_post", {post_address: postAddr})
  t.ok(gotPost.Ok)
  
  let linked = await alice.callSync('blog', 'main', 'recommend_post', {
    post_address: postAddr, 
    agent_address: alice.agentId
  })
  console.log("linked: ", linked)
  t.equal(linked.Ok, null)

  const recommendedPosts = alice.call('blog', 'main', 'my_recommended_posts', {})
  console.log("recommendedPosts", recommendedPosts)
  console.log('agent addresses: ', alice.agentId, alice.agentId)

  t.equal(recommendedPosts.Ok.addresses.length, 1)
})

scenario.runTape('alice create & publish post -> tash recommend to self', async (t, {alice, tash}) => {
  t.plan(4)
  const content = "Holo world...2"
  const in_reply_to = null
  const params = { content: content, in_reply_to }
  const postResult = await alice.callSync("blog", "main", "create_post", params)
  const postAddr = postResult.Ok
  t.ok(postAddr, `error: ${postResult}`)

  const gotPost = tash.call("blog", "main", "get_post", {post_address: postAddr})
  t.ok(gotPost.Ok)
  
  let linked = await tash.callSync('blog', 'main', 'recommend_post', {
    post_address: postAddr, 
    agent_address: tash.agentId
  })
  console.log("linked: ", linked)
  t.equal(linked.Ok, null)

  const recommendedPosts = tash.call('blog', 'main', 'my_recommended_posts', {})
  console.log("recommendedPosts", recommendedPosts)
  console.log('agent addresses: ', alice.agentId, tash.agentId)

  t.equal(recommendedPosts.Ok.addresses.length, 1)
})

scenario.runTape('create & publish post -> recommend to other agent', async (t, {alice, tash}) => {
  t.plan(4)
  const content = "Holo world...3"
  const in_reply_to = null
  const params = { content: content, in_reply_to }
  const postResult = await alice.callSync("blog", "main", "create_post", params)
  const postAddr = postResult.Ok
  t.ok(postAddr, `error: ${postResult}`)

  const gotPost = tash.call("blog", "main", "get_post", {post_address: postAddr})
  t.ok(gotPost.Ok)
  
  let linked = await alice.callSync('blog', 'main', 'recommend_post', {
    post_address: postAddr, 
    agent_address: tash.agentId
  })
  console.log("linked: ", linked)
  t.equal(linked.Ok, null)
  
  const recommendedPosts = tash.call('blog', 'main', 'my_recommended_posts', {})
  console.log("recommendedPosts", recommendedPosts)
  console.log('agent addresses: ', alice.agentId, tash.agentId)

  t.equal(recommendedPosts.Ok.addresses.length, 1)
})
