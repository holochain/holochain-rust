// use riker_default::DefaultModel;
use riker::actors::*;
use snowflake;
use hash_table::pair::Pair;
use error::HolochainError;
// use riker_patterns::ask::ask;
// use futures::executor::block_on;
use actor::SYS;
use actor::Protocol;
use actor::AskSelf;

#[derive(Clone, Debug)]
pub enum ChainProtocol {
    SetTopPair(Option<Pair>),
    SetTopPairResult(Result<Option<Pair>, HolochainError>),

    GetTopPair,
    GetTopPairResult(Option<Pair>),
}

// lazy_static! {
//     // @TODO Riker docs say make only one actor system per application but this seems weird advice
//     // if that were true, how could actors be in crates?
//     // if that were true, how could we have domain specific protocols?
//     pub static ref CHAIN_SYS: ActorSystem<ChainProtocol> = {
//         let model: DefaultModel<ChainProtocol> = DefaultModel::new();
//         ActorSystem::new(&model).unwrap()
//     };
// }

// impl Into<ActorMsg<ChainProtocol>> for ChainProtocol {
//     fn into(self) -> ActorMsg<ChainProtocol> {
//         ActorMsg::User(self)
//     }
// }

/// anything that can be asked of Chain and block on responses
/// needed to support implementing ask on upstream ActorRef from riker
pub trait AskChain {
    // fn ask(&self, message: Protocol) -> Protocol;
    fn set_top_pair(&self, &Option<Pair>) -> Result<Option<Pair>, HolochainError>;
    fn get_top_pair(&self) -> Option<Pair>;
}

impl AskChain for ActorRef<Protocol> {
    // fn ask(&self, message: Protocol) -> Protocol {
    //     let a = ask(&(*SYS), self, message);
    //     block_on(a).unwrap()
    // }

    fn set_top_pair(&self, pair: &Option<Pair>) -> Result<Option<Pair>, HolochainError> {
        let response = self.ask(Protocol::SetTopPair(pair.clone()));
        unwrap_to!(response => Protocol::SetTopPairResult).clone()
    }

    fn get_top_pair(&self) -> Option<Pair> {
        let response = self.ask(Protocol::GetTopPair);
        unwrap_to!(response => Protocol::GetTopPairResult).clone()
    }
}

pub struct ChainActor {
    top_pair: Option<Pair>,
}

impl ChainActor {
    pub fn new() -> ChainActor {
        ChainActor {
            top_pair: None,
        }
    }

    pub fn actor() -> BoxActor<Protocol> {
        Box::new(ChainActor::new())
    }

    pub fn props() -> BoxActorProd<Protocol> {
        Props::new(Box::new(ChainActor::actor))
    }

    pub fn new_ref() -> ActorRef<Protocol> {
        SYS
            .actor_of(
                ChainActor::props(),
                &snowflake::ProcessUniqueId::new().to_string(),
            )
            .unwrap()
    }
}

impl Actor for ChainActor {
    type Msg = Protocol;

    fn receive(
        &mut self,
        context: &Context<Self::Msg>,
        message: Self::Msg,
        sender: Option<ActorRef<Self::Msg>>,
    ) {
        println!("rec: {:?}", message);
        sender.try_tell(
            // deliberately exhaustively matching here, don't give into _ temptation
            match message {
                Protocol::SetTopPair(p) => {
                    self.top_pair = p;
                    Protocol::SetTopPairResult(Ok(self.top_pair.clone()))
                },
                Protocol::SetTopPairResult(_) => unreachable!(),

                Protocol::GetTopPair => {
                    println!("get top pair inner");
                    let ret = self.top_pair.clone();
                    println!("inner: {:?}", ret);
                    Protocol::GetTopPairResult(ret)
                },
                Protocol::GetTopPairResult(_) => unreachable!(),

                _ => unreachable!(),
            },
            Some(context.myself()),
        )
        .unwrap();
    }

}
