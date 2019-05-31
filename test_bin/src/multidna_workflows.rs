use crate::three_workflows::setup_three_nodes;
use constants::*;
use holochain_core_types::cas::content::Address;
use holochain_net::{
    connection::{json_protocol::JsonProtocol, NetResult},
    tweetlog::TWEETLOG,
};
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
    alex.send_direct_message(&CAMILLE_AGENT_ID, ENTRY_CONTENT_1.clone());
    let res = camille
        .wait_json(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::HandleSendMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(*ENTRY_CONTENT_1, msg.content);
    log_i!("Send messages on DNA A COMPLETE \n\n\n");

    // Billy should not receive it
    alex.send_direct_message(&BILLY_AGENT_ID, ENTRY_CONTENT_1.clone());
    let res =
        billy.wait_json_with_timeout(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))), 1000);
    assert!(res.is_none());

    // Send messages on DNA B
    // ======================
    alex.set_current_dna(&DNA_ADDRESS_B);
    billy.set_current_dna(&DNA_ADDRESS_B);
    camille.set_current_dna(&DNA_ADDRESS_B);

    // Billy should receive it
    alex.send_direct_message(&BILLY_AGENT_ID, ENTRY_CONTENT_2.clone());
    let res = billy
        .wait_json(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::HandleSendMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(*ENTRY_CONTENT_2, msg.content);

    // Camille should not receive it
    alex.send_direct_message(&CAMILLE_AGENT_ID, ENTRY_CONTENT_2.clone());
    let res =
        camille.wait_json_with_timeout(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))), 1000);
    assert!(res.is_none());
    log_i!("Send messages on DNA B COMPLETE \n\n\n");

    // Send messages on DNA C
    // ======================
    alex.set_current_dna(&DNA_ADDRESS_C);
    billy.set_current_dna(&DNA_ADDRESS_C);
    camille.set_current_dna(&DNA_ADDRESS_C);

    // Camille should receive it
    camille.send_direct_message(&BILLY_AGENT_ID, ENTRY_CONTENT_3.clone());
    let res = billy
        .wait_json(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))))
        .unwrap();
    log_i!("#### got: {:?}", res);
    let msg = match res {
        JsonProtocol::HandleSendMessage(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(*ENTRY_CONTENT_3, msg.content);

    // Alex should not receive it
    camille.send_direct_message(&ALEX_AGENT_ID, ENTRY_CONTENT_3.clone());
    let res =
        alex.wait_json_with_timeout(Box::new(one_is!(JsonProtocol::HandleSendMessage(_))), 1000);
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
    alex.author_entry(&ENTRY_ADDRESS_1, vec![ENTRY_CONTENT_1.clone()], true)?;

    // Camille asks for that data
    let query_data = camille.request_entry(ENTRY_ADDRESS_1.clone());

    // Alex sends that data back to the network
    alex.reply_to_HandleQueryEntry(&query_data).unwrap();

    // Camille should receive requested data
    let result = camille
        .wait_json(Box::new(one_is!(JsonProtocol::QueryEntryResult(_))))
        .unwrap();
    log_i!("got QueryEntryResult: {:?}", result);

    // Billy asks for data on unknown DNA
    let query_data = billy.request_entry(ENTRY_ADDRESS_1.clone());

    // Alex sends that data back to the network
    alex.reply_to_HandleQueryEntry(&query_data).unwrap();

    // Billy might receive FailureResult
    let result =
        billy.wait_json_with_timeout(Box::new(one_is!(JsonProtocol::FailureResult(_))), 1000);
    log_i!("got FailureResult: {:?}", result);

    // Done
    Ok(())
}

//// this is all debug code, no need to track code test coverage
//#[cfg_attr(tarpaulin, skip)]
//pub fn meta_test(
//    alex: &mut TestNode,
//    billy: &mut TestNode,
//    camille: &mut TestNode,
//    can_connect: bool,
//) -> NetResult<()> {
//    // Setup
//    setup_three_nodes(alex, billy, camille, &DNA_ADDRESS_A, can_connect)?;
//    multi_untrack(vec![billy], &[&DNA_ADDRESS_A])?;
//    multi_track(vec![alex, billy], &[&DNA_ADDRESS_B])?;
//    multi_track(vec![billy, camille], &[&DNA_ADDRESS_C])?;
//    alex.set_current_dna(&DNA_ADDRESS_B);
//    billy.set_current_dna(&DNA_ADDRESS_B);
//
//    // wait for gossip
//    let _msg_count = billy.listen(200);
//
//    // Alex publishs entry & meta on DNA B
//    alex.author_entry(&ENTRY_ADDRESS_3, vec![ENTRY_CONTENT_3.clone()], true)?;
//    alex.author_meta(
//        &ENTRY_ADDRESS_3,
//        &META_LINK_ATTRIBUTE.to_string(),
//        &META_LINK_CONTENT_3,
//        true,
//    )?;
//
//    // Billy requests that meta
//    let fetch_meta = billy.request_meta(ENTRY_ADDRESS_3.clone(), META_LINK_ATTRIBUTE.to_string());
//    // Alex sends HandleFetchMetaResult message
//    alex.reply_to_HandleFetchMeta(&fetch_meta)?;
//    // billy should receive requested metadata
//    let result = billy
//        .wait(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))))
//        .unwrap();
//    log_i!("got GetMetaResult: {:?}", result);
//    let meta_data = unwrap_to!(result => JsonProtocol::FetchMetaResult);
//    assert_eq!(meta_data.entry_address, ENTRY_ADDRESS_3.clone());
//    assert_eq!(meta_data.attribute, META_LINK_ATTRIBUTE.clone());
//    assert_eq!(meta_data.content_list.len(), 1);
//    assert_eq!(meta_data.content_list[0], META_LINK_CONTENT_3.clone());
//
//    // Camille requests that meta
//    let _fetch_meta =
//        camille.request_meta(ENTRY_ADDRESS_3.clone(), META_LINK_ATTRIBUTE.to_string());
//    // Camille should not receive requested metadata
//    let result =
//        camille.wait_with_timeout(Box::new(one_is!(JsonProtocol::FetchMetaResult(_))), 1000);
//    log_i!("got GetMetaResult: {:?}", result);
//
//    // XXX - currently in-memory and mock return none here
//    //       and realmode returns a result with no meta items.
//    if !result.is_none() {
//        let result = result.unwrap();
//        let meta_data = unwrap_to!(result => JsonProtocol::FetchMetaResult);
//        assert_eq!(meta_data.entry_address, ENTRY_ADDRESS_3.clone());
//        assert_eq!(meta_data.attribute, META_LINK_ATTRIBUTE.clone());
//        assert_eq!(meta_data.content_list.len(), 0);
//    }
//
//    // Done
//    Ok(())
//}
