use lib3h::rrdht_util::*;
use lib3h_crypto_api::CryptoSystem;

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
    if node_count < 50 {
        return true;
    }

    // divide up the space so on average data will be stored by 50 nodes
    let dist: f64 = 4294967295.0 / (node_count as f64 / 50.0);

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
        let crypto: Box<dyn CryptoSystem> =
            Box::new(SodiumCryptoSystem::new().set_pwhash_interactive());

        let mut nodes = Vec::new();

        let mut min = 50;
        let mut max = 50;
        let mut count = 0;
        let mut mean = 0.0;

        // simulate a 10,000 node network, growing 10 nodes at a time
        for _ in 0..1000 {
            for _ in 0..10 {
                nodes.push(anything_to_location(&crypto, &gen_id(&crypto)));
            }

            // simulate storing 100 bits of data in this network
            for _ in 0..100 {
                let data = anything_to_location(&crypto, &gen_data_addr(&crypto));

                let mut store_count = 0;

                // go through all the nodes
                for agent_loc in nodes.iter() {
                    if naive_sharding_should_store(*agent_loc, data, nodes.len() as u64) {
                        store_count += 1;
                    }
                }

                if nodes.len() < 50 {
                    // if we have less than 50 nodes
                    // make sure all nodes store all data
                    assert_eq!(store_count, nodes.len());
                } else {
                    // if we have > 50 nodes,
                    // assert that a reasonable number of nodes store the data
                    assert!(store_count > 15);
                    assert!(store_count < 100);

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
