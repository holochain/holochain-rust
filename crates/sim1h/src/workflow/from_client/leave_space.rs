use crate::{
    dht::bbdht::{dynamodb::client::Client, error::BbDhtResult},
    trace::{tracer, LogContext},
};
use lib3h_protocol::{data_types::SpaceData, protocol::ClientToLib3hResponse};

/// no-op
pub fn leave_space(
    log_context: &LogContext,
    _client: &Client,
    _leave_space_data: &SpaceData,
) -> BbDhtResult<ClientToLib3hResponse> {
    tracer(&log_context, "leave_space");
    // leave space is a no-op in a simulation
    Ok(ClientToLib3hResponse::LeaveSpaceResult)
}

#[cfg(test)]
pub mod tests {

    use crate::{
        dht::bbdht::dynamodb::client::local::local_client, space::fixture::space_data_fresh,
        trace::tracer, workflow::from_client::leave_space::leave_space,
    };
    use lib3h_protocol::protocol::ClientToLib3hResponse;

    #[test]
    fn leave_space_test() {
        let log_context = "leave_space_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let space_data = space_data_fresh();

        tracer(&log_context, "check response");
        match leave_space(&log_context, &local_client, &space_data) {
            Ok(ClientToLib3hResponse::LeaveSpaceResult) => {
                tracer(&log_context, "👌");
            }
            Ok(v) => {
                panic!("bad Ok {:?}", v);
            }
            Err(err) => {
                panic!("Err {:?}", err);
            }
        }
    }

}
