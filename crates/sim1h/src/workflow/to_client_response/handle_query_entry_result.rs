use crate::{
    trace::{tracer, LogContext},
    workflow::state::Sim1hState,
};
use lib3h_protocol::{
    data_types::QueryEntryResultData, protocol::ClientToLib3hResponse,
};

impl Sim1hState {
    /// Response to a `HandleQueryEntry` request
    pub fn handle_query_entry_result(
        &mut self,
        log_context: &LogContext,
        data: &QueryEntryResultData,
    ) {
        tracer(
            &log_context,
            &format!("handle_query_entry_result {:?}", data),
        );

        // If the original query request originated from this core, then mirror it
        // back as the response -- because in sim1h, the only person who can fulfill your query
        // request is yourself, ultimately. Query requests are intercepted, they trigger Holds
        // on entry aspects, which triggers a HandleQuery request, which ultimately triggers
        // this mirroring you're seeing here.
        if data.space_address == self.space_hash
            && data.requester_agent_id == self.agent_id
        {
            self.client_response_outbox
                .push(ClientToLib3hResponse::QueryEntryResult(data.clone()))
        }
    }
}
