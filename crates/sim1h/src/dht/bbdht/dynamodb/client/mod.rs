pub mod fixture;
pub mod local;

use dynomite::{
    dynamodb::DynamoDbClient,
    retry::{Policy, RetryingDynamoDb},
    Retries,
};
use rusoto_core::Region;
use std::time::Duration;

pub type Client = RetryingDynamoDb<DynamoDbClient>;

pub fn client(region: Region) -> Client {
    DynamoDbClient::new(region).with_retries(Policy::Exponential(10, Duration::from_millis(100)))
}

pub fn client_from_endpoint(endpoint: String, region: String) -> Client {
    client(Region::Custom {
        name: region,
        endpoint,
    })
}

#[cfg(test)]
pub mod test {
    use crate::{dht::bbdht::dynamodb::client::client, trace::tracer};
    use rusoto_core::region::Region;

    #[test]
    fn client_smoke_test() {
        let log_context = "client_smoke_test";

        tracer(&log_context, "smoke test");
        client(Region::SaEast1);
    }
}
