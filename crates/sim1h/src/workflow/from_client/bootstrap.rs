use crate::dht::bbdht::dynamodb::account::describe_limits;
use crate::dht::bbdht::dynamodb::client::Client;
use crate::dht::bbdht::error::BbDhtResult;
use crate::trace::tracer;
use crate::trace::LogContext;
use lib3h_protocol::protocol::ClientToLib3hResponse;

/// check database connection
/// optional
pub fn bootstrap(log_context: &LogContext, client: &Client) -> BbDhtResult<ClientToLib3hResponse> {
    tracer(&log_context, "bootstrap");
    // touch the database to check our connection is good
    describe_limits(&log_context, &client)?;
    Ok(ClientToLib3hResponse::BootstrapSuccess)
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::dht::bbdht::dynamodb::client::fixture::bad_client;
    use crate::dht::bbdht::dynamodb::client::local::local_client;
    use crate::trace::tracer;
    use crate::workflow::from_client::bootstrap::bootstrap;

    #[test]
    fn bootstrap_test() {
        let log_context = "bootstrap_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();

        // success
        match bootstrap(&log_context, &local_client) {
            Ok(ClientToLib3hResponse::BootstrapSuccess) => {}
            Ok(v) => {
                panic!("Bad Ok {:?}", v);
            }
            Err(err) => {
                panic!("Err {:?}", err);
            }
        }
    }

    #[test]
    fn bootstrap_bad_client_test() {
        let log_context = "bootstrap_bad_client_test";

        tracer(&log_context, "fixtures");
        let bad_client = bad_client();

        // fail
        match bootstrap(&log_context, &bad_client) {
            Err(_) => {
                tracer(&log_context, "👌");
            }
            Ok(v) => {
                panic!("bad Ok {:?}", v);
            }
        };
    }

}
