use actor::Protocol;
use cas::content::AddressableContent;
use cas::storage::ContentAddressableStorage;
use riker::actors::*;
use error::HolochainError;
use cas::content::Address;

pub struct StorageActor {
    inner: ContentAddressableStorage + Send,
}

impl StorageActor {
    pub fn new<CAS: ContentAddressableStorage>(inner: CAS) {
        StorageActor {
            inner
        }
    }
}

pub struct AddWrapper {
    inner: Box<AddressableContent + Send>,
}

impl AddWrapper {
    pub fn new<AC: AddressableContent + Send>(inner: AC) {
        AddWrapper{
            inner
        }
    }
}

pub trait AskStorage: ContentAddressableStorage {}

impl AskStorage for ActorRef<Protocol<AddressableContent>> {}

impl ContentAddressableStorage for ActorRef<Protocol<AddressableContent>> {
    fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
        let response = self.block_on_ask(Protocol::CasAdd(AddWrapper::new(content.clone())))?;
        unwrap_to!(response => Protocol::CasAddResult).clone()
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        let response = self.block_on_ask(Protocol::CasContains(address.clone()))?;
        unwrap_to!(response => Protocol::CasContainsResult).clone()
    }

    fn fetch<C: AddressableContent>(&self, address: &Address) -> Result<Option<C>, HolochainError> {
        let response = self.block_on_ask(Protocol::CasFetch(address.clone()))?;
        unwrap_to!(response => Protocol::CasFetchResult).clone()
    }
}

impl Actor for StorageActor {
    type Msg = Protocol<AddressableContent>;

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
                    Protocol::CasAdd(content) => {
                        Protocol::CasAddResult(self.inner.add(content))
                    }

                    Protocol::CasContains(address) => {
                        Protocol::CasContainsResult(self.inner.contains(address))
                    }

                    Protocol::CasFetch(address) => {
                        Protocol::CasFetchResult(self.inner.fetch(address))
                    }

                    _ => unreachable!(),
                },
                Some(context.myself()),
            )
            .expect("failed to tell StorageActor sender");
    }
}
