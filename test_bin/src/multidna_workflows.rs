use crate::three_workflows::setup_three_nodes;
use constants::*;
use holochain_net::{connection::NetResult, tweetlog::TWEETLOG};
use holochain_persistence_api::cas::content::Address;
use lib3h_protocol::protocol_server::Lib3hServerProtocol;
use p2p_node::test_node::TestNode;

/// Have multiple nodes track multiple dnas
#[cfg_attr(tarpaulin, skip)]
pub fn multi_track(mut nodes: Vec<&mut TestNode>, dnas: &[&Address]) -> NetResult<()> {
    for dna in dnas {
        for mut node in &mut nodes {
            node.track_dna(dna, false)?;
        }
    }
    Ok(())
}

/// Have multiple nodes untrack multiple dnas
#[cfg_attr(tarpaulin, skip)]
pub fn multi_untrack(mut nodes: Vec<&mut TestNode>, dnas: &[&Address]) -> NetResult<()> {
    for mut node in &mut nodes {
        for dna in dnas {
            node.untrack_dna(dna)?;
        }
    }
    Ok(())
}

/// Have 3 nodes.
/// Each pair of nodes are tracking the same DNA
/// Pairs should be able to send messages between them.
/// Non-pairs should not.
#[cfg_attr(tarpaulin, skip)]
pub fn send_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    camille: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    setup_three_nodes(alex, billy, camille, &DNA_ADDRESS_A, can_connect)?;
    multi_untrack(vec![billy], &[&DNA_ADDRESS_A])?;
    multi_track(vec![alex, billy], &[&DNA_ADDRESS_B])?;
    multi_track(vec![billy, camille], &[&DNA_ADDRESS_C])?;

    // Send messages on DNA A
    // ======================
    alex.set_current_dna(&DNA_ADDRESS_A);
    billy.set_current_dna(&DNA_ADDRESS_A);
    camille.set_current_dna(&DNA_ADDRESS_A);

    // Camille should receive it
    alex.send_direct_message(&CAMILLE_AGENT_ID, ASPECT_CONTENT_1.clone());
    let res = camille
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::HandleSendDirectMessage(_))))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        Lib3hServerProtocol::HandleSendDirectMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(*ASPECT_CONTENT_1, msg.content);
    log_i!("Send messages on DNA A COMPLETE \n\n\n");

    // Billy should not receive it
    alex.send_direct_message(&BILLY_AGENT_ID, ASPECT_CONTENT_1.clone());
    let res = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleSendDirectMessage(_))),
        1000,
    );
    assert!(res.is_none());

    // Send messages on DNA B
    // ======================
    alex.set_current_dna(&DNA_ADDRESS_B);
    billy.set_current_dna(&DNA_ADDRESS_B);
    camille.set_current_dna(&DNA_ADDRESS_B);

    // Billy should receive it
    alex.send_direct_message(&BILLY_AGENT_ID, ASPECT_CONTENT_2.clone());
    let res = billy
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::HandleSendDirectMessage(_))))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        Lib3hServerProtocol::HandleSendDirectMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(*ASPECT_CONTENT_2, msg.content);

    // Camille should not receive it
    alex.send_direct_message(&CAMILLE_AGENT_ID, ASPECT_CONTENT_2.clone());
    let res = camille.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleSendDirectMessage(_))),
        1000,
    );
    assert!(res.is_none());
    log_i!("Send messages on DNA B COMPLETE \n\n\n");

    // Send messages on DNA C
    // ======================
    alex.set_current_dna(&DNA_ADDRESS_C);
    billy.set_current_dna(&DNA_ADDRESS_C);
    camille.set_current_dna(&DNA_ADDRESS_C);

    // Camille should receive it
    camille.send_direct_message(&BILLY_AGENT_ID, ASPECT_CONTENT_3.clone());
    let res = billy
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::HandleSendDirectMessage(_))))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        Lib3hServerProtocol::HandleSendDirectMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(*ASPECT_CONTENT_3, msg.content);

    // Alex should not receive it
    camille.send_direct_message(&ALEX_AGENT_ID, ASPECT_CONTENT_3.clone());
    let res = alex.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::HandleSendDirectMessage(_))),
        1000,
    );
    assert!(res.is_none());
    log_i!("Send messages on DNA C COMPLETE \n\n\n");

    // Done
    Ok(())
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
pub fn dht_test(
    alex: &mut TestNode,
    billy: &mut TestNode,
    camille: &mut TestNode,
    can_connect: bool,
) -> NetResult<()> {
    // Setup
    setup_three_nodes(alex, billy, camille, &DNA_ADDRESS_A, can_connect)?;
    multi_untrack(vec![billy], &[&DNA_ADDRESS_A])?;
    multi_track(vec![alex, billy], &[&DNA_ADDRESS_B])?;
    multi_track(vec![billy, camille], &[&DNA_ADDRESS_C])?;
    alex.set_current_dna(&DNA_ADDRESS_A);

    // wait for gossip
    let _msg_count = alex.listen(200);

    // Alex publish data on the network
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ASPECT_CONTENT_1.clone()], true)?;
    // Gossip might ask us for the data
    let maybe_fetch_a = alex.wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::HandleFetchEntry(_))));
    if let Some(fetch_a) = maybe_fetch_a {
        let fetch = unwrap_to!(fetch_a => Lib3hServerProtocol::HandleFetchEntry);
        let _ = alex.reply_to_HandleFetchEntry(&fetch).unwrap();
    }
    // Check if both nodes are asked to store it
    let _ = camille.wait_lib3h(Box::new(one_is!(
        Lib3hServerProtocol::HandleStoreEntryAspect(_)
    )));

    // Camille asks for that data
    let query_data = camille.request_entry(ENTRY_ADDRESS_1.clone());

    // #fullsync
    // camille sends that data back to the network
    camille.reply_to_HandleQueryEntry(&query_data).unwrap();

    // Camille should receive requested data
    let result = camille
        .wait_lib3h(Box::new(one_is!(Lib3hServerProtocol::QueryEntryResult(_))))
        .unwrap();
    log_i!("got QueryEntryResult: {:?}", result);

    // Billy asks for data on unknown DNA
    let query_data = billy.request_entry(ENTRY_ADDRESS_1.clone());

    // #fullsync
    // Billy sends that data back to the network
    let _ = billy.reply_to_HandleQueryEntry(&query_data);

    // Billy might receive FailureResult
    let result = billy.wait_lib3h_with_timeout(
        Box::new(one_is!(Lib3hServerProtocol::FailureResult(_))),
        1000,
    );
    log_i!("got FailureResult: {:?}", result);

    // Done
    Ok(())
}
