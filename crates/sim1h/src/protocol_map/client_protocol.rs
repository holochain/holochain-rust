use lib3h_protocol::protocol_client::Lib3hClientProtocol;


/// mimic lib3h::engine::real_engine::serve_Lib3hClientProtocol
pub fn serve_Lib3hClientProtocol(client_msg: Lib3hClientProtocol) {
    debug!("serving: {:?}", client_msg);

    /// docs for all sequences at:
    /// https://hackmd.io/Rag5au4dQfm1CtcjOK7y5w
    match protocol {
        Lib3hClientProtocol::Shutdown => {
            // ** do nothing **
            // this is a hangover from n3h
        },

        // this doesn't do anything standalone
        Lib3hClientProtocol::SuccessResult(generic_result_data) => { generic_result_data; },

        // this doesn't do anything standalone
        Lib3hClientProtocol::FailureResult(generic_result_data) => { generic_result_data; },

        // https://hackmd.io/Rag5au4dQfm1CtcjOK7y5w#Connect
        Lib3hClientProtocol::Connect(connect_data) => {
            // ??CHECK??
            // - is this still needed in ghost actor land?

            // short term:
            // this is A:
            // if B in table then connected success!
            // return Lib3hServerProtocol::Connected(peerB_uri) to A core
            // else
            // return Lib3hServerProtocol::FailureResult to A core

            // long term:
            // this is A:
            // check if B enabled in space
            //  if not Lib3hClientProtocol::FailureResult to A core
            // put A -> B Lib3hClientProtocol::Connect(peerB_uri) in db
            connect_data;
        },

        // https://hackmd.io/Rag5au4dQfm1CtcjOK7y5w#JoinSpace
        Lib3hClientProtocol::JoinSpace(space_data) => {
            //   create table if not exists
            //   enable self in table and dirty poll
            // return if no db error
            //   - Lib3hSeverProtocol::SuccessResult to core
            //   - Lib3hServerProtocol::HandleGetAuthoringEntryList to core
            //   - Lib3hServerProtocol::HandleGetGossipingEntryList to core
            // return Lib3hClientProtocol::FailureResult if there is a db error
        },
        Lib3hClientProtocol::LeaveSpace(space_data) => {
            // short term:
            // disable self in table
            // flush all dirty polls

            // long term: cancel outstanding polling e.g. for connections or whatever
        },
        Lib3hClientProtocol::SendDirectMessage(direct_message_data) => {
            // this is A:
            // we put the message in the database for B
            // start a dirty poll for Lib3hClientProtocol::HandleSendDirectMessageResult
        },
        Lib3hClientProtocol::HandleSendDirectMessageResult(direct_message_data) => {
            // this is A:
            // dirty poll the db to see if there is a pending result from B
            // stop the dirty poll
            // pass it on to core
         },

        Lib3hClientProtocol::FetchEntry(fetch_entry_data) => {
            // this is A:
            // get from db
            //   send Lib3hSeverProtocol::FetchEntryResult to A core
         },
        Lib3hClientProtocol::HandleFetchEntryResult(fetch_entry_result_data) => {
            // short term:
            // never going to happen

            fetch_entry_result_data;
        },

        Lib3hClientProtocol::PublishEntry(provided_entry_data) => {

            // this is A:
            // put in db
            // this includes both an Entry and EntryAspects

             provided_entry_data;

        },
        Lib3hClientProtocol::HoldEntry(provided_entry_data) => {
            // short term:
            // this never happens we assume local validation is enough

            // long term:
            // some kind of query to do neighbourhoods

            provided_entry_data;
        },
        Lib3hClientProtocol::QueryEntry(query_entry_data) => {
            // ?? CHECK ??
            // - see what this is about

            query_entry_data;
        },
        Lib3hClientProtocol::HandleQueryEntryResult(query_entry_result_data) => {
            // short term:
            // this never happens
             query_entry_result_data; },

        Lib3hClientProtocol::HandleGetAuthoringEntryListResult(entry_list_data) => {
            // ??CHECK??
            // - what is needed short/long term?
            entry_list_data; },
        Lib3hClientProtocol::HandleGetGossipingEntryListResult(entry_list_data) => {
            // short term:
            // this never happens
            entry_list_data; },
    }
}
