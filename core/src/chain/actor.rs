use riker::actors::*;

#[derive(Clone, Debug)]
pub struct ChainActor;

impl ChainActor {
    pub fn new() -> ChainActor {
        ChainActor {}
    }
}

impl Actor for ChainActor {
    type Msg = String;

    fn receive(
        &mut self,
        context: &Context<Self::Msg>,
        message: Self::Msg,
        sender: Option<ActorRef<Self::Msg>>,
    ) {
        println!("received {}", message);
    }
}

impl ChainActor {
    pub fn actor() -> BoxActor<String> {
        Box::new(ChainActor)
    }

    pub fn props() -> BoxActorProd<String> {
        Props::new(Box::new(ChainActor::actor))
    }
}
