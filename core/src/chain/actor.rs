use holochain_core_types::{
    actor::{AskSelf, Protocol, SYS},
    chain_header::ChainHeader,
    error::HolochainError,
};
use riker::actors::*;
use snowflake;

/// anything that can be asked of Chain and block on responses
/// needed to support implementing ask on upstream ActorRef from riker
pub trait AskChain {
    /// Protocol::SetTopChainHeader -> Protocol::SetTopChainHeaderResult
    fn set_top_chain_header(
        &self,
        &Option<ChainHeader>,
    ) -> Result<Option<ChainHeader>, HolochainError>;
    /// Protocol::GetTopChainHeader -> Protocol::GetTopChainHeaderResult
    fn top_chain_header(&self) -> Result<Option<ChainHeader>, HolochainError>;
}

impl AskChain for ActorRef<Protocol> {
    fn set_top_chain_header(
        &self,
        chain_header: &Option<ChainHeader>,
    ) -> Result<Option<ChainHeader>, HolochainError> {
        let response = self.block_on_ask(Protocol::SetTopChainHeader(chain_header.clone()))?;
        unwrap_to!(response => Protocol::SetTopChainHeaderResult).clone()
    }

    fn top_chain_header(&self) -> Result<Option<ChainHeader>, HolochainError> {
        let response = self.block_on_ask(Protocol::GetTopChainHeader)?;
        Ok(unwrap_to!(response => Protocol::GetTopChainHeaderResult).clone())
    }
}

pub struct ChainActor {
    top_chain_header: Option<ChainHeader>,
}

impl ChainActor {
    /// returns a new ChainActor struct
    /// internal use for riker, use new_ref instead
    fn new() -> ChainActor {
        ChainActor {
            top_chain_header: None,
        }
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
                    // set the top chain header to the value passed
                    Protocol::SetTopChainHeader(chain_header) => {
                        self.top_chain_header = chain_header;
                        Protocol::SetTopChainHeaderResult(Ok(self.top_chain_header.clone()))
                    }

                    // evaluates to the current top chain header
                    Protocol::GetTopChainHeader => {
                        let ret = self.top_chain_header.clone();
                        Protocol::GetTopChainHeaderResult(ret)
                    }

                    _ => unreachable!(),
                },
                Some(context.myself()),
            )
            .expect("failed to tell ChainActor sender");
    }
}
/*
#[cfg(test)]
pub mod tests {
    use holochain_cas_implementations::actor::Protocol;
    use chain::{
        actor::{AskChain, ChainActor},
        chain_header::tests::{test_chain_header_a, test_chain_header_b},
    };
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
    /// can set and get top chain headers through the chain actor
    fn test_round_trip() {
        let chain_actor = test_chain_actor();

        assert_eq!(
            None,
            chain_actor
                .top_chain_header()
                .expect("could not get top chain header from chain actor")
        );

        let chain_header_a = test_chain_header_a();
        chain_actor
            .set_top_chain_header(&Some(chain_header_a.clone()))
            .expect("could not set top chain_header a");

        assert_eq!(
            Some(chain_header_a.clone()),
            chain_actor
                .top_chain_header()
                .expect("could not get top chain_header from chain actor")
        );

        let chain_header_b = test_chain_header_b();
        chain_actor
            .set_top_chain_header(&Some(chain_header_b.clone()))
            .expect("could not set top chain_header b");

        assert_eq!(
            Some(chain_header_b.clone()),
            chain_actor
                .top_chain_header()
                .expect("could not get top chain_header from chain actor")
        );
    }

}
*/
