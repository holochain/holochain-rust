# Remove Link

`Recommended` : read link_entries documentation before reading this documentation.

Remove Link is the process of marking a link as deleted in the DHT. For this process to happen, the remove_link process has to get all links that correspond with the target. This is each time a Link is commited, it is linked against the address of the link entry rather than the target. For this reason, it is why we have to run a get_links to obtain all the corresponding addresses that contain that target and set those links as deleted. This allows for links with the same target to be added after they have deleted e.g social media friend that has been removed and readded. The RemoveLink ends up adding an entry of LinkRemove to signifiy that this link has been deleted and if a similar link of LinkAdd with the same hash is added after, it will act as a tombstone and not acknowledge it.

# Validation

`System Validation`

A system validation also takes place when a remove_link is executed. Before a remove_link is executed, we have to make sure that the link base address exists. This takes place in the system validation, after the system validation is complete, zome validation runs, see below.

`Zome Validation`

A Zome Validation always takes place on each remove_link that is executed after it passes the system validation. These are rules that can be defined by the zome. Thus, the Zome Developer can choose approve link deletion using their defined criteria.


