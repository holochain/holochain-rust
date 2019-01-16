
const path = require('path')
const sleep = require('sleep')
const test = require('tape')
const { pollFor } = require('./util')

const { Config, Container } = require('../../nodejs_container')

const dnaPath = path.join(__dirname, "../dist/app_spec.hcpkg")
const aliceName = "alice"
const tashName = "tash"

// IIFE to keep config-only stuff out of test scope
const container = (() => {
  const agentAlice = Config.agent(aliceName)
  const agentTash = Config.agent(tashName)

  const dna = Config.dna(dnaPath)

  const instanceAlice = Config.instance(agentAlice, dna)
  const instanceTash = Config.instance(agentTash, dna)

  const containerConfig = Config.container([instanceAlice, instanceTash])
  return new Container(containerConfig)
})()

// Initialize the Container
container.start()

const alice = container.makeCaller(aliceName, dnaPath)
const tash = container.makeCaller(tashName, dnaPath)


// run the following three tests ONE AT A TIME. 
// The first fails, the second and third pass.

test('calling get_links before link_entries makes a difference', (t) => {

  const get1 = alice.call("blog", "main", "my_posts", {})
  t.ok(get1.Ok)
  sleep.sleep(1)

  const create1 = alice.call("blog", "main", "create_post", {content: 'hi'})
  t.ok(create1.Ok)
  sleep.sleep(1)

  const get2 = alice.call("blog", "main", "my_posts", {})
  t.ok(get2.Ok)

  t.equal(get2.Ok.addresses.length, 1)
  t.end()
})


test('calling get_links twice in a row is different than calling it once', (t) => {
  // This test is exactly the same as the previous one, but calls my_posts twice in a row.
  // This makes the links come through the second time.

  const get1 = alice.call("blog", "main", "my_posts", {})
  t.ok(get1.Ok)
  sleep.sleep(1)

  const create1 = alice.call("blog", "main", "create_post", {content: 'hi'})
  t.ok(create1.Ok)
  sleep.sleep(1)

  alice.call("blog", "main", "my_posts", {})
  const get2 = alice.call("blog", "main", "my_posts", {})
  t.ok(get2.Ok)

  t.equal(get2.Ok.addresses.length, 1)
  t.end()
})

test('not calling get_links in the beginning helps', (t) => {

  const create1 = alice.call("blog", "main", "create_post", {content: 'hi'})
  t.ok(create1.Ok)
  sleep.sleep(1)

  const get1 = alice.call("blog", "main", "my_posts", {})
  t.ok(get1.Ok)

  t.equal(get1.Ok.addresses.length, 1)
  t.end()
})

//////////////////////////////////
//////////////////////////////////

test('alice create & publish post -> recommend own post to self', async (t) => {
  t.plan(4)
  const content1 = "Holo world...1"
  const in_reply_to = null
  const params = { content: content1, in_reply_to }

  
  // TODO can go away after scenario-api merge
  const numInitialRecommendedPosts = alice.call('blog', 'main', 'my_recommended_posts', {}).Ok.addresses.length

  const postAddr = alice.call("blog", "main", "create_post", params).Ok
  t.ok(postAddr)

  const gotPost = await pollFor(
    () => alice.call("blog", "main", "get_post", {post_address: postAddr})
  ).catch(t.fail)
  t.ok(gotPost.Ok)
  
  let linked = alice.call('blog', 'main', 'recommend_post', {
    post_address: postAddr, 
    agent_address: alice.agentId
  })
  console.log("linked: ", linked)
  t.equal(linked.Ok, null)
  
  sleep.sleep(1)
  
  const recommendedPosts = alice.call('blog', 'main', 'my_recommended_posts', {})
  console.log("recommendedPosts", recommendedPosts)
  console.log('agent addresses: ', alice.agentId, alice.agentId)

  t.equal(recommendedPosts.Ok.addresses.length, numInitialRecommendedPosts + 1)
})

test('alice create & publish post -> tash recommend to self', async (t) => {
  t.plan(4)
  const content1 = "Holo world...2"
  const in_reply_to = null
  const params = { content: content1, in_reply_to }
  
  // TODO can go away after scenario-api merge
  const numInitialRecommendedPosts = tash.call('blog', 'main', 'my_recommended_posts', {}).Ok.addresses.length

  const postAddr = alice.call("blog", "main", "create_post", params).Ok
  t.ok(postAddr)

  const gotPost = await pollFor(
    () => tash.call("blog", "main", "get_post", {post_address: postAddr})
  ).catch(t.fail)
  t.ok(gotPost.Ok)
  
  let linked = tash.call('blog', 'main', 'recommend_post', {
    post_address: postAddr, 
    agent_address: tash.agentId
  })
  console.log("linked: ", linked)
  t.equal(linked.Ok, null)
  
  sleep.sleep(1)

  const recommendedPosts = tash.call('blog', 'main', 'my_recommended_posts', {})
  console.log("recommendedPosts", recommendedPosts)
  console.log('agent addresses: ', alice.agentId, tash.agentId)

  t.equal(recommendedPosts.Ok.addresses.length, numInitialRecommendedPosts + 1)
})

test('create & publish post -> recommend to other agent', async (t) => {
  t.plan(4)
  const content1 = "Holo world...3"
  const in_reply_to = null
  const params = { content: content1, in_reply_to }
  
  // TODO can go away after scenario-api merge
  const numInitialRecommendedPosts = tash.call('blog', 'main', 'my_recommended_posts', {}).Ok.addresses.length

  const postAddr = alice.call("blog", "main", "create_post", params).Ok
  t.ok(postAddr)

  const gotPost = await pollFor(
    () => tash.call("blog", "main", "get_post", {post_address: postAddr})
  ).catch(t.fail)
  t.ok(gotPost.Ok)
  
  let linked = alice.call('blog', 'main', 'recommend_post', {
    post_address: postAddr, 
    agent_address: tash.agentId
  })
  console.log("linked: ", linked)
  t.equal(linked.Ok, null)

  sleep.sleep(1)
  
  const recommendedPosts = tash.call('blog', 'main', 'my_recommended_posts', {})
  console.log("recommendedPosts", recommendedPosts)
  console.log('agent addresses: ', alice.agentId, tash.agentId)

  t.equal(recommendedPosts.Ok.addresses.length, numInitialRecommendedPosts + 1)
})
