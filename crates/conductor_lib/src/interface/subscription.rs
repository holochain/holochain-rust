use interface::ConductorApiBuilder;
use jsonrpc_core::{futures, types::params::Params, Value};
use jsonrpc_pubsub::{Subscriber, SubscriptionId};
use snowflake::ProcessUniqueId;

impl ConductorApiBuilder {
    pub fn with_signal_subscriptions(mut self) -> Self {
        // TODO: use a real channel, where the receiver is hooked up to a place where
        // these Sinks can be stored and kept track of
        let (tx, _rx) = crossbeam_channel::unbounded();
        let tx_clone = tx.clone();

        self.handler.io.add_subscription(
            "signal/consistency",
            (
                "subscribe/signal/consistency",
                move |_: Params, _, subscriber: Subscriber| {
                    let id = SubscriptionId::String(ProcessUniqueId::new().to_string());
                    let sink = subscriber
                        .assign_id(id)
                        .expect("Could not assign subscriber ID");
                    tx_clone.send(sink).unwrap();
                    // subscriptions.insert(id, sink);
                },
            ),
            (
                "unsubscribe/signal/consistency",
                |_id: SubscriptionId, _| futures::future::ok(Value::Bool(true)),
            ),
        );
        self
    }
}
