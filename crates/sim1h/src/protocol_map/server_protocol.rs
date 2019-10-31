/// mimic lib3h::engine::real_engine::serve_Lib3hClientProtocol
pub fn serve_Lib3hServerProtocol(client_msg: Lib3hClientProtocol) {
    debug!("serving: {:?}", client_msg);

    /// docs for all sequences at:
    /// https://hackmd.io/Rag5au4dQfm1CtcjOK7y5w
    match protocol {
        pub enum Lib3hServer(InFromNetwork)Protocol {

            // this doesn't do anything standalone
            SuccessResult(GenericResultData),

            // this doesn't do anything standalone
            FailureResult(GenericResultData),

            Connected(ConnectedData) {

                // short term:
                // this never happens! it's just returned to A if B in db

                // ???CHECK???
                // - what to do when a connection fails?
                // - what happens if A and B disagree on connection state (B thinks it is connected at the exact moment A times out)

                // long term:
                // this is B:
                // something in B sees the A -> B Lib3hClientProtocol::Connect(peerB_uri) in the db
                //   check if A dirty polled recently in space
                //     if not Lib3hClientProtocol::FailureResult to B core
                //   B puts A -> B Lib3hServerProtocol::Connected(peerB_uri) in the db
                //   B sends A -> B Lib3hServerProtocol::Connected(peerA_uri) to B's core

                // this is A:
                // something in A sees the A -> B Lib3hServerProtocol::Connected(peerB_uri) within TIMEOUT
                //   A sends Lib3hServerProtocol::Connected(peerB_uri) to A's core
                // else there was a TIMEOUT so
                //   A sends Lib3hClientProtocol::FailureResult to A's core

            }

            Disconnected(DisconnectedData) {
                // short term:
                // - this can't happen because connect doesn't happen
                // - at least it would be a no-op
            }

            SendDirectMessageResult(DirectMessageData), {
                // this is B:
                // put the result for A back in the db for A's dirty poll to discover

            }

            HandleSendDirectMessage(DirectMessageData), {
                // this B:
                // something has put a pending message for us in the db
                // forward it on to core
            }

            FetchEntryResult(FetchEntryResultData) {
                // this is A
                // this is what is returned to core with an entry in it hopefully
            }

            HandleFetchEntry(FetchEntryData) {
                // short term:
                // this never happens because data magically always comes from "someone else"

                // long term:
                // trigger this when people query the db
            }

            HandleStoreEntryAspect(StoreEntryAspectData) {
                // short term:
                // never going to happen as aspects live in the db
            }

            HandleDropEntry(DropEntryData) {
                // short term:
                // not going to happen as there are no arcs

                // ?? CHECK ??
                // - confirm whether we require this for deletion of entry aspects
            }

            HandleQueryEntry(QueryEntryData) {
                // ?? CHECK ??
            }

            QueryEntryResult(QueryEntryResultData) {
                // ?? CHECK ??
            }

            HandleGetAuthoringEntryList(GetListData) {
                // ?? CHECK ??
            }

            HandleGetGossipingEntryList(GetListData) {
                // this doesn't happen
            }

            // n3h specific
            Terminated,

            // n3h specific
            P2pReady,
        }
    }
}
