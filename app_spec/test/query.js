const path = require('path')
const { Config, Conductor, Scenario } = require('../../nodejs_conductor')
Scenario.setTape(require('tape'))

const dnaPath = path.join(__dirname, "../dist/app_spec.dna.json")
const dna = Config.dna(dnaPath, 'app-spec')
const agentAlice = Config.agent("alice")

const instanceAlice = Config.instance(agentAlice, dna)

const scenario = new Scenario([instanceAlice], { debugLog: true })

scenario.runTape('query posts based using time index', async (t, {alice}) => {
    let date = new Date(); //Get current timestamp - post should be linked to the same timestamps
    date.toISOString();
    let year = date.getFullYear().toString();
    let month = date.getMonth().toString();
    let day = date.getDate().toString();
    let hour = date.getHours().toString();
    
    //create post which will create index links on following timestamp anchors: year, month, day, hour
    const create1 = await alice.callSync("blog","create_post", {content: 'hi', in_reply_to: null})
    t.ok(create1)

    //Get the hash of the month timestamp we want to start our query from
    const timestamp_base_hash_month = await alice.callSync("blog", "get_timestamp_address", {timestamp: month, time_type: "Month"})
    t.equal(timestamp_base_hash_month.Ok, 'QmaNY1DwVqCA5S1PEgzBgSxcVo7XU6oFxMjf4ABpo7i74h')
  
    //Query where month is current month, year, day and hour. This would return all posts which where posted in given year, month, day, hour
    const query = await alice.callSync("blog", "query_posts", {base: timestamp_base_hash_month.Ok, query_string: year+"<Time:Y>:"+day+"<Time:D>:"+hour+"<Time:H>"})
    t.ok(query)

    //Query to return all posts posted in given month and year
    const query2 = await alice.callSync("blog", "query_posts", {base: timestamp_base_hash_month.Ok, query_string: year+"<Time:Y>"})
    t.ok(query2)
  })