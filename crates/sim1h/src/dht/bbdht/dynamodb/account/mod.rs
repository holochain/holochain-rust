use crate::{
    dht::bbdht::{dynamodb::client::Client, error::BbDhtResult},
    trace::{tracer, LogContext},
};
use rusoto_dynamodb::{DescribeLimitsOutput, DynamoDb};

pub fn describe_limits(
    log_context: &LogContext,
    client: &Client,
) -> BbDhtResult<DescribeLimitsOutput> {
    tracer(&log_context, "describe_limits");
    Ok(client.describe_limits().sync()?)
}

#[cfg(test)]
pub mod tests {

    use crate::{
        dht::bbdht::dynamodb::{
            account::describe_limits,
            client::{fixture::bad_client, local::local_client},
        },
        trace::tracer,
    };

    #[test]
    fn describe_limits_ok_test() {
        let log_context = "describe_limits_ok_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();

        // describe limits
        assert!(describe_limits(&log_context, &local_client).is_ok());
    }

    #[test]
    fn describe_limits_bad_test() {
        let log_context = "describe_limits_bad_test";

        tracer(&log_context, "fixtures");
        let bad_client = bad_client();

        // fail to describe limits
        assert!(describe_limits(&log_context, &bad_client).is_err());
    }

}
