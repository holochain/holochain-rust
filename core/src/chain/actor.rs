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

/// anything that can be asked of Chain and block on responses
/// needed to support implementing ask on upstream ActorRef from riker
pub trait AskChain {
    // fn ask(&self, message: Protocol) -> Protocol;
    fn set_top_pair(&self, &Option<Pair>) -> Result<Option<Pair>, HolochainError>;
    fn top_pair(&self) -> Option<Pair>;
}

impl AskChain for ActorRef<Protocol> {
    fn set_top_pair(&self, pair: &Option<Pair>) -> Result<Option<Pair>, HolochainError> {
        let response = self.ask(Protocol::SetTopPair(pair.clone()));
        unwrap_to!(response => Protocol::SetTopPairResult).clone()
    }

    fn top_pair(&self) -> Option<Pair> {
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
        sender.try_tell(
            // deliberately exhaustively matching here, don't give into _ temptation
            match message {
                Protocol::SetTopPair(p) => {
                    self.top_pair = p;
                    Protocol::SetTopPairResult(Ok(self.top_pair.clone()))
                },
                Protocol::SetTopPairResult(_) => unreachable!(),

                Protocol::GetTopPair => {
                    let ret = self.top_pair.clone();
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
