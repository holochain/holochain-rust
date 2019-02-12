use jsonrpc_pubsub::{PubSubHandler, Session};
use std::sync::Arc;

pub type RpcHandler = PubSubHandler<Option<Arc<Session>>>;
