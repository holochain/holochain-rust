use lib3h::rrdht_util::*;
use lib3h_crypto_api::CryptoSystem;

const REDUNDANT_COUNT: u64 = 50;

#[allow(clippy::borrowed_box)]
/// ack - lib3h can only convert agent_ids to locations right now
/// work around this in a dorky manner
pub fn anything_to_location(crypto: &Box<dyn CryptoSystem>, anything: &str) -> Location {
    match calc_location_for_id(crypto, anything) {
        Ok(loc) => loc,
        Err(_) => {
            let mut hash = crypto.buf_new_insecure(crypto.hash_sha256_bytes());
            let r: Box<dyn lib3h_crypto_api::Buffer> = Box::new(anything.as_bytes().to_vec());
            crypto.hash_sha256(&mut hash, &r).unwrap();
            calc_location_for_id(
                crypto,
                &hcid::HcidEncoding::with_kind("hcs0")
                    .unwrap()
                    .encode(&hash)
                    .unwrap(),
            )
            .unwrap()
        }
    }
}

/// implement a super simple sharding algorithm
/// to distribute data when node counts go > 50
pub fn naive_sharding_should_store(
    agent_loc: Location,
    data_addr_loc: Location,
    node_count: u64,
) -> bool {
    // if there are < 50 nodes, everyone should store everything
    if node_count <= REDUNDANT_COUNT {
        return true;
    }

    // divide up the space so on average data will be stored by 50 nodes
    let dist: f64 = ARC_LENGTH_MAX as f64 / (node_count as f64 / REDUNDANT_COUNT as f64);

    // determine if this specific piece of data should be stored by this node
    agent_loc.forward_distance_to(data_addr_loc) < dist as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib3h_sodium::SodiumCryptoSystem;

    // generate a test agent id (HcS)
    fn gen_id(crypto: &Box<dyn CryptoSystem>) -> String {
        let enc = hcid::HcidEncoding::with_kind("hcs0").unwrap();
        let mut key = crypto.buf_new_insecure(32);
        crypto.randombytes_buf(&mut key).unwrap();
        enc.encode(&key).unwrap()
    }

    // generate a test data address (HcA)
    fn gen_data_addr(crypto: &Box<dyn CryptoSystem>) -> String {
        let mut data = crypto.buf_new_insecure(32);
        crypto.randombytes_buf(&mut data).unwrap();
        let mut addr = crypto.buf_new_insecure(32);
        crypto.hash_sha256(&mut addr, &data).unwrap();
        let enc = hcid::HcidEncoding::with_kind("hca0").unwrap();
        enc.encode(&addr).unwrap()
    }

    #[test]
    fn it_should_safely_distribute_data() {
        let thread_cont = std::sync::Arc::new(std::sync::Mutex::new(true));
        let mut hash_threads = Vec::new();

        let (id_send, id_recv) = crossbeam_channel::bounded::<Location>(20);
        let (addr_send, addr_recv) = crossbeam_channel::bounded::<Location>(200);

        for _ in 0..8 {
            let id_send_clone = id_send.clone();
            let addr_send_clone = addr_send.clone();
            let cont = thread_cont.clone();
            hash_threads.push(std::thread::spawn(move || {
                let crypto: Box<dyn CryptoSystem> =
                    Box::new(SodiumCryptoSystem::new().set_pwhash_interactive());
                let mut id = None;
                let mut addr = None;
                loop {
                    {
                        if !*cont.lock().unwrap() {
                            break;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    if id.is_none() {
                        id = Some(anything_to_location(&crypto, &gen_id(&crypto)));
                    }
                    if addr.is_none() {
                        addr = Some(anything_to_location(&crypto, &gen_data_addr(&crypto)));
                    }
                    match id_send_clone.try_send(id.unwrap()) {
                        Ok(_) => (),
                        Err(crossbeam_channel::TrySendError::Full(eid)) => {
                            id = Some(eid);
                        }
                        _ => panic!("send fail"),
                    }
                    match addr_send_clone.try_send(addr.unwrap()) {
                        Ok(_) => (),
                        Err(crossbeam_channel::TrySendError::Full(ea)) => {
                            addr = Some(ea);
                        }
                        _ => panic!("send fail"),
                    }
                }
            }));
        }

        //let crypto: Box<dyn CryptoSystem> =
        //    Box::new(SodiumCryptoSystem::new().set_pwhash_interactive());

        let mut nodes = Vec::new();

        let mut min = REDUNDANT_COUNT;
        let mut max = REDUNDANT_COUNT;
        let mut count = 0;
        let mut mean = 0.0;

        // simulate a 10,000 node network, growing 10 nodes at a time
        for _ in 0..1000 {
            for _ in 0..10 {
                let id_loc = id_recv.recv().unwrap();
                //println!("id: {}", *id_loc);
                nodes.push(id_loc);
            }

            println!("NODE COUNT: {}", nodes.len());

            // simulate storing 100 bits of data in this network
            for _ in 0..100 {
                let data_loc = addr_recv.recv().unwrap();
                //println!("data: {}", *data_loc);

                let mut store_count = 0_u64;

                // go through all the nodes
                for agent_loc in nodes.iter() {
                    if naive_sharding_should_store(*agent_loc, data_loc, nodes.len() as u64) {
                        store_count += 1;
                    }
                }

                println!(" store - {} - {}", store_count, *data_loc);

                if (nodes.len() as u64) < REDUNDANT_COUNT {
                    // if we have less than 50 nodes
                    // make sure all nodes store all data
                    assert_eq!(store_count, nodes.len() as u64);
                } else {
                    // if we have > 50 nodes,
                    // assert that a reasonable number of nodes store the data
                    if store_count < 15 {
                        let dist: f64 = ARC_LENGTH_MAX as f64 / (nodes.len() as f64 / REDUNDANT_COUNT as f64) * 100.0 / ARC_LENGTH_MAX as f64;
                        println!("-- NOT STORING ENOUGH --");
                        println!("-- dist: {}% --", dist as u64);
                        println!("-- data loc: {}% --", u64::from((data_loc.0).0) * 100 / ARC_LENGTH_MAX);
                        for agent_loc in nodes.iter() {
                            println!("  - agent loc: {}% - {}", u64::from((agent_loc.0).0) * 100 / ARC_LENGTH_MAX, naive_sharding_should_store(*agent_loc, data_loc, nodes.len() as u64));
                        }
                        panic!("store count < 15: {}", store_count);
                    }
                    if store_count >= 100 {
                        println!("-- STORING TOO MUCH --");
                        println!("-- data loc: {} --", *data_loc);
                        for agent_loc in nodes.iter() {
                            println!("  - agent loc: {}", **agent_loc);
                        }
                        panic!("store count >= 100: {}", store_count);
                    }
                    //assert!(store_count > 15, format!("got: {} out of {} nodes", store_count, nodes.len()));
                    //assert!(store_count < 100);

                    if store_count < min {
                        min = store_count;
                    }
                    if store_count > max {
                        max = store_count;
                    }
                    mean = (mean * count as f64 + store_count as f64) / (count as f64 + 1.0);
                    count += 1;
                }
            }
        }

        *thread_cont.lock().unwrap() = false;

        for t in hash_threads.drain(..) {
            t.join().unwrap();
        }

        // gives values like
        // count: 99600 (because we track tests with < 50 nodes)
        // min: 25
        // max: 78
        // mean: 49.99037148594384
        println!(
            "count: {}\nmin: {}\nmax: {}\nmean: {}",
            count, min, max, mean
        );
    }
}
