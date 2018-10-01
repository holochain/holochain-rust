# What is the TPS (Transactions Per Second) on Holochain?

> Holochain doesn't have a set TPS (transactions per second) like other blockchain-based or blockchain-derived projects might because there's central point through which all transactions must pass. Instead, Holochain is a generalized protocol for distributed computing.

It's common to ask a blockchain project, "How much can your technology handle? What's its TPS?" This is because nearly all of these projects are built around the limiting idea of a global ledger.

But you are not asking, how many posts per second Facebook can do. Why? Because there is no technical problem, adding more servers to Facebook's data center (only maybe monetary problems).

You are not asking how many emails per second the internet can handle, because there is no single bottleneck for email-sending, like there would be with a centralized approach.

Why are we seeing a transaction limit with blockchain networks? Because blockchain in a strange way marries a decentralized p2p network of nodes with the logical notion of one absolute truth, i.e. the blockchain being one big decentralized database of transactions. It tries to maintain this way of thinking about apps that we are used to from centralized servers. It forces every node into the same "consensus". That is implemented by having everybody share and validate everything. That does work, and maybe there are few usecases (like a global naming system maybe?) where it might be advantageous.. but applying that for everything is nonsensical.

Holochain is not forcing such a model. Instead it allows for building applications that are like email. The application is rather like a protocol, or grammar, or (I prefer this language) like a dance. If you know the dance (If you have a copy of the validation rules of the app) you can tell who else is dancing that dance and who is not. The difference between Holochain and something like email is that (similarly to blockhain) Holochain is applying 1. cryptographic signatures and 2. tamper proof hash-chains (hence Holo*chain*) so that you can build a distributed system you can trust in. You know it is impossible (I'd rather say: very very hard) to game somebody. This so far was only possible by having trusted authorities like banks or Facebook.

So, Holochain as an app framework does not pose any limit of transactions per second because there is no place where all transactions have to go through. It is like asking, "how many words can humanity speak per second?" Well, with every human being born, that number increases. Same for Holochain.

