# Remove Link

`Recommended` : read link_entries documentation before reading this documentation.

Remove Link is the process of marking a link as deleted in the DHT. For this process to happen, the remove_link first has to get all like hashes that share the same base,tag,link type and target. The remove link process will then mark all of these links as deleted in the process. The reason for this is that links that share the same base, tag, link_type and target do not necesarrily produce the same link hash. In order for the delete to work all corresponding link hashes would have to be marked as deleted in the DHT
# Validation

`System Validation`

A system validation also takes place when a remove_link is executed. Before a remove_link is executed, we have to make sure that the link base address exists. This takes place in the system validation, after the system validation is complete, zome validation runs, see below.

`Zome Validation`

A Zome Validation always takes place on each remove_link that is executed after it passes the system validation. These are rules that can be defined by the zome. Thus, the Zome Developer can choose approve link deletion using their defined criteria.


