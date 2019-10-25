//! fixtures for test clients

use crate::dht::bbdht::dynamodb::client::client;
use crate::dht::bbdht::dynamodb::client::Client;
use rusoto_core::region::Region;

/// the region means nothing for a local install
const BAD_REGION: &str = "badbad";
/// the endpoint needs to be explicitly set to hit the local database
const BAD_ENDPOINT: &str = "http://example.com";

pub fn bad_region() -> Region {
    Region::Custom {
        name: BAD_REGION.into(),
        endpoint: BAD_ENDPOINT.into(),
    }
}

pub fn bad_client() -> Client {
    client(bad_region())
}

#[cfg(test)]
pub mod tests {
    use crate::dht::bbdht::dynamodb::client::fixture::bad_client;
    use crate::dht::bbdht::dynamodb::client::fixture::bad_region;
    use crate::dht::bbdht::dynamodb::client::fixture::BAD_ENDPOINT;
    use crate::dht::bbdht::dynamodb::client::fixture::BAD_REGION;

    use crate::trace::tracer;
    use rusoto_core::region::Region;

    #[test]
    /// check the value is what we want
    fn bad_region_test() {
        let log_context = "bad_region_test";

        tracer(&log_context, "compare values");
        let bad_region = bad_region();
        assert_eq!(
            Region::Custom {
                name: BAD_REGION.into(),
                endpoint: BAD_ENDPOINT.into(),
            },
            bad_region
        );
    }

    #[test]
    fn bad_client_smoke_test() {
        let log_context = "bad_client_smoke_test";

        tracer(&log_context, "smoke test");
        bad_client();
    }
}
