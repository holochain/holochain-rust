//! settings and convenience fns for a local client

use crate::dht::bbdht::dynamodb::client::{client, Client};
use rusoto_core::region::Region;

/// the region means nothing for a local install
pub const LOCAL_REGION: &str = "us-east-1";
/// the endpoint needs to be explicitly set to hit the local database
pub const LOCAL_ENDPOINT: &str = "http://localhost:8000";

pub fn local_region() -> Region {
    Region::Custom {
        name: LOCAL_REGION.into(),
        endpoint: LOCAL_ENDPOINT.into(),
    }
}

pub fn local_client() -> Client {
    client(local_region())
}

#[cfg(test)]
pub mod tests {
    use crate::dht::bbdht::dynamodb::client::local::{
        local_client, local_region, LOCAL_ENDPOINT, LOCAL_REGION,
    };

    use crate::trace::tracer;
    use rusoto_core::region::Region;

    #[test]
    /// check the value is what we want
    fn local_region_test() {
        let log_context = "local_region_test";

        tracer(&log_context, "compare values");
        let region = local_region();
        assert_eq!(
            Region::Custom {
                name: LOCAL_REGION.into(),
                endpoint: LOCAL_ENDPOINT.into(),
            },
            region
        );
    }

    #[test]
    fn local_client_smoke_test() {
        let log_context = "local_client_smoke_test";

        tracer(&log_context, "smoke test");
        local_client();
    }
}
