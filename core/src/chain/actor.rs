use actor::{AskSelf, Protocol, SYS};
use error::HolochainError;
use hash_table::pair::Pair;
use riker::actors::*;
use snowflake;

/// anything that can be asked of Chain and block on responses
/// needed to support implementing ask on upstream ActorRef from riker
pub trait AskChain {
    /// Protocol::SetTopPair -> Protocol::SetTopPairResult
    fn set_top_pair(&self, &Option<Pair>) -> Result<Option<Pair>, HolochainError>;
    /// Protocol::GetTopPair -> Protocol::GetTopPairResult
    fn top_pair(&self) -> Result<Option<Pair>, HolochainError>;
}

impl AskChain for ActorRef<Protocol> {
    fn set_top_pair(&self, pair: &Option<Pair>) -> Result<Option<Pair>, HolochainError> {
        let response = self.block_on_ask(Protocol::SetTopPair(pair.clone()))?;
        unwrap_to!(response => Protocol::SetTopPairResult).clone()
    }

    fn top_pair(&self) -> Result<Option<Pair>, HolochainError> {
        let response = self.block_on_ask(Protocol::GetTopPair)?;
        Ok(unwrap_to!(response => Protocol::GetTopPairResult).clone())
    }
}

pub struct ChainActor {
    top_pair: Option<Pair>,
}

impl ChainActor {
    /// returns a new ChainActor struct
    /// internal use for riker, use new_ref instead
    fn new() -> ChainActor {
        ChainActor { top_pair: None }
    }

    /// actor() for riker
    fn actor() -> BoxActor<Protocol> {
        Box::new(ChainActor::new())
    }

    /// props() for riker
    fn props() -> BoxActorProd<Protocol> {
        Props::new(Box::new(ChainActor::actor))
    }

    /// returns a new actor ref for a new ChainActor in the main actor system
    pub fn new_ref() -> ActorRef<Protocol> {
        SYS.actor_of(
            ChainActor::props(),
            &snowflake::ProcessUniqueId::new().to_string(),
        ).expect("could not create ChainActor in actor system")
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
        sender
            .try_tell(
                match message {
                    // set the top pair to the value passed
                    Protocol::SetTopPair(p) => {
                        self.top_pair = p;
                        Protocol::SetTopPairResult(Ok(self.top_pair.clone()))
                    }

                    // evaluates to the current top pair
                    Protocol::GetTopPair => {
                        let ret = self.top_pair.clone();
                        Protocol::GetTopPairResult(ret)
                    }

                    _ => unreachable!(),
                },
                Some(context.myself()),
            )
            .expect("failed to tell ChainActor sender");
    }
}

#[cfg(test)]
pub mod tests {
    use actor::Protocol;
    use chain::actor::{AskChain, ChainActor};
    use hash_table::pair::tests::{test_pair_a, test_pair_b};
    use riker::actors::*;

    /// dummy chain actor reference
    pub fn test_chain_actor() -> ActorRef<Protocol> {
        ChainActor::new_ref()
    }

    #[test]
    /// smoke test new refs
    fn test_new_ref() {
        test_chain_actor();
    }

    #[test]
    /// can set and get top pairs through the chain actor
    fn test_round_trip() {
        let chain_actor = test_chain_actor();

        assert_eq!(None, chain_actor.top_pair().expect("could not get top pair from chain actor"));

        let pair_a = test_pair_a();
        chain_actor
            .set_top_pair(&Some(pair_a.clone()))
            .expect("could not set top pair a");

        assert_eq!(Some(pair_a.clone()), chain_actor.top_pair().expect("could not get top pair from chain actor"));

        let pair_b = test_pair_b();
        chain_actor
            .set_top_pair(&Some(pair_b.clone()))
            .expect("could not set top pair b");

        assert_eq!(Some(pair_b.clone()), chain_actor.top_pair().expect("could not get top pair from chain actor"));
    }

}
