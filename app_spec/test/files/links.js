module.exports = scenario => {

    scenario('delete_post', async (s, t, { alice, bob }) => {

        //creates a simple link with alice as author with initial chain header
        await alice.app.callSync("simple", "create_link",
          { "base":alice.app.agentId, "target": "Posty" }
        )
      
      
        //creates a simple link with bob as author with different chain header
        await bob.app.callSync("simple", "create_link",
          { "base":alice.app.agentId, "target": "Posty" }
        )
      
        //get all created links so far alice
        const alice_posts = await bob.app.call("simple", "get_my_links",
          { "base": alice.app.agentId,"status_request" : "Live" }
        )
      
      
        //expect two links from alice
        t.ok(alice_posts.Ok)
        t.equal(alice_posts.Ok.links.length,2 );
      
        //get all created links so far for bob
        const bob_posts = await bob.app.call("simple", "get_my_links",
          { "base": alice.app.agentId,"status_request" : "Live" }
        )
      
      
        //expected two links from bob
        t.ok(bob_posts.Ok)
        t.equal(bob_posts.Ok.links.length,2 );
      
        //alice removes both links
        await alice.app.callSync("simple", "delete_link", { "base":alice.app.agentId, "target": "Posty" })
      
        // get links from bob
        const bob_agent_posts_expect_empty = await bob.app.call("simple", "get_my_links",{ "base": alice.app.agentId,"status_request" : "Live" })
        //get links from alice
        const alice_agent_posts_expect_empty = await alice.app.call("simple", "get_my_links",{ "base": alice.app.agentId,"status_request" : "Live" })
      
        //bob expects zero links
        t.ok(bob_agent_posts_expect_empty.Ok)
        t.equal(bob_agent_posts_expect_empty.Ok.links.length, 0);
        //alice expects zero alice
        t.ok(alice_agent_posts_expect_empty.Ok)
        t.equal(alice_agent_posts_expect_empty.Ok.links.length, 0);
      
      
        //different chain hash up to this point so we should be able to create a link with the same data
        await alice.app.callSync("simple", "create_link",{ "base":alice.app.agentId, "target": "Posty" })
      
        //get alice posts
        const alice_posts_not_empty = await bob.app.call("simple", "get_my_links",{ "base": alice.app.agentId,"status_request" : "Live" })
      
         //expect 1 post
        t.ok(alice_posts_not_empty.Ok)
        t.equal(alice_posts_not_empty.Ok.links.length, 1);
      
      
      })
      

      scenario('delete_post_with_bad_link', async (s, t, { alice, bob }) => {

        const result_bob_delete = await bob.app.callSync("blog", "delete_post", {
          "content": "Bad"
        })
      
        // bad in_reply_to is an error condition
        t.ok(result_bob_delete.Err)
        t.notOk(result_bob_delete.Ok)
        const error = JSON.parse(result_bob_delete.Err.Internal)
        t.deepEqual(error.kind, { ErrorGeneric: "Target for link not found" })
        t.ok(error.file)
        t.ok(error.line)
      })

      scenario('get_links_crud', async (s, t, { alice, bob }) => {

        //commits an entry and creates two links for alice
        await alice.app.callSync("simple", "create_link",
          { "base": alice.app.agentId ,"target": "Holo world" }
        );
        const alice_result = await alice.app.callSync("simple", "create_link",
        { "base": alice.app.agentId ,"target": "Holo world 2" }
        );
      
        //get posts for alice from alice
        const alice_posts_live= await alice.app.call("simple","get_my_links",
        {
          "base" : alice.app.agentId,"status_request":"Live"
        })
        console.log("alice posts" + JSON.stringify(alice_posts_live));
      
        //get posts for alice from bob
        const bob_posts_live= await bob.app.call("simple","get_my_links",
        {
          "base" : alice.app.agentId,
          "status_request":"Live"
        })
      
        //make sure all our links are live and they are two of them
        t.equal(2,alice_posts_live.Ok.links.length);
        t.equal("live",alice_posts_live.Ok.links[0].status);
        t.equal("live",alice_posts_live.Ok.links[1].status);
        t.equal(2,bob_posts_live.Ok.links.length);
        t.equal("live",bob_posts_live.Ok.links[0].status);
        t.equal("live",bob_posts_live.Ok.links[1].status);
      
        ////delete the holo world post from the links alice created
        await alice.app.callSync("simple","delete_link",
        {
          "base" : alice.app.agentId,
          "target" : "Holo world"
        });
      
        //get all posts with a deleted status from bob
        const bob_posts_deleted = await bob.app.call("simple","get_my_links",
        {
          "base" : alice.app.agentId,
          "status_request" : "Deleted"
        });
      
        // get all posts with a deleted status from alice
        const alice_posts_deleted = await alice.app.call("simple","get_my_links",
        {
          "base" : alice.app.agentId,
          "status_request" : "Deleted"
        });
      
        //make sure only 1 is returned and it has a status of deleted
        t.equal(1,alice_posts_deleted.Ok.links.length);
        t.equal(1,bob_posts_deleted.Ok.links.length);
        t.equal("deleted",alice_posts_deleted.Ok.links[0].status);
        t.equal("deleted",bob_posts_deleted.Ok.links[0].status);
      
        //get all posts from the agent
        const bob_posts_all = await bob.app.call("simple","get_my_links",
        {
          "base" : alice.app.agentId,
          "status_request" : "All"
      
        });
        const alice_posts_all = await alice.app.call("simple","get_my_links",
        {
          "base" : alice.app.agentId,
          "status_request" : "All"
        });
      
        //make sure we get two links with the first one being a live link and the second one being a deleted link
        t.equal(2,alice_posts_all.Ok.links.length);
        t.equal("live",alice_posts_all.Ok.links[0].status);
        t.equal("deleted",alice_posts_all.Ok.links[1].status);
        t.equal(2,bob_posts_all.Ok.links.length);
        t.equal("live",bob_posts_all.Ok.links[0].status);
        t.equal("deleted",bob_posts_all.Ok.links[1].status);
      
      
      })
      
      scenario('get_links_crud_count', async (s, t, { alice, bob }) => {
      
        //commits an entry and creates two links for alice
        await alice.app.callSync("simple", "create_link_with_tag",
          { "base": alice.app.agentId ,"target": "Holo world","tag":"tag" }
        );
      
        //commit an entry with other tag
        await alice.app.callSync("simple", "create_link_with_tag",
        { "base": alice.app.agentId ,"target": "Holo world", "tag":"differen" }
         );
        
        await alice.app.callSync("simple", "create_link_with_tag",
        { "base": alice.app.agentId ,"target": "Holo world 2","tag":"tag" });
      
        //get posts for alice from alice
        const alice_posts_live= await alice.app.call("simple","get_my_links_count",
        {
          "base" : alice.app.agentId,
          "status_request":"Live",
          "tag":"tag"
        })
      
        //get posts for alice from bob
        const bob_posts_live= await bob.app.call("simple","get_my_links_count",
        {
          "base" : alice.app.agentId,
          "status_request":"Live",
          "tag":"tag"
        })
      
       
        
        //make sure count equals to 2
        t.equal(2,alice_posts_live.Ok.count);
        t.equal(2,bob_posts_live.Ok.count);
      
        const bob_posts_live_diff_tag= await bob.app.call("simple","get_my_links_count",
        {
          "base" : alice.app.agentId,
          "status_request":"Live",
          "tag":"differen"
        })
      
        t.equal(1,bob_posts_live_diff_tag.Ok.count);
      
      
        ////delete the holo world post from the links alice created
        await alice.app.callSync("simple","delete_link_with_tag",
        {
          "base" : alice.app.agentId,
          "target" : "Holo world",
          "tag":"tag"
        });
      
        //get all bob posts
        const bob_posts_deleted = await bob.app.call("simple","get_my_links_count",
        {
          "base" : alice.app.agentId,
          "status_request" : "Deleted",
          "tag":"tag"
        });
      
        // get all posts with a deleted status from alice
        const alice_posts_deleted = await alice.app.call("simple","get_my_links_count",
        {
          "base" : alice.app.agentId,
          "status_request" : "Deleted",
          "tag":"tag"
        });
      
        //make sure count is equal to 1
        t.equal(1,alice_posts_deleted.Ok.count);
        t.equal(1,bob_posts_deleted.Ok.count);
      
        const bob_posts_deleted_diff_tag= await bob.app.call("simple","get_my_links_count",
        {
          "base" : alice.app.agentId,
          "status_request":"Live",
          "tag":"differen"
        })
      
        t.equal(1,bob_posts_deleted_diff_tag.Ok.count);
      
      })

      scenario('get_sources_after_same_link', async (s, t, { alice, bob }) => {

        await bob.app.callSync("blog", "create_post_with_agent",
          { "agent_id": alice.app.agentId ,"content": "Holo world", "in_reply_to": null }
        );
        await bob.app.callSync("blog", "create_post_with_agent",
        { "agent_id": alice.app.agentId ,"content": "Holo world", "in_reply_to": null }
        );
      
        const alice_posts = await bob.app.call("blog","authored_posts_with_sources",
        {
          "agent" : alice.app.agentId
        });
        const bob_posts = await alice.app.call("blog","authored_posts_with_sources",
        {
          "agent" : alice.app.agentId
        });
      
        t.equal(bob.app.agentId,alice_posts.Ok.links[0].headers[0].provenances[0][0]);
        t.equal(bob.app.agentId,bob_posts.Ok.links[0].headers[0].provenances[0][0]);
      
      })

    
    

      
      scenario('get_sources_from_link', async (s, t, { alice, bob }) => {
      
        await alice.app.callSync("blog", "create_post", {
          "content": "Holo world", "in_reply_to": null
        });
      
        await bob.app.callSync("blog", "create_post", {
          "content": "Another one", "in_reply_to": null
        });
        const alice_posts = await bob.app.call("blog","authored_posts_with_sources", {
          "agent" : alice.app.agentId
        });
      
        const bob_posts = await alice.app.call("blog","authored_posts_with_sources", {
          "agent" : bob.app.agentId
        });
      
        t.equal(bob_posts.Ok.links.length, 1)
        t.equal(bob.app.agentId,bob_posts.Ok.links[0].headers[0].provenances[0][0]);
        t.equal(alice_posts.Ok.links.length, 1)
        t.equal(alice.app.agentId,alice_posts.Ok.links[0].headers[0].provenances[0][0]);
      
      })
      

}