use hdk::prelude::*;

fn handle_sum(num1: u32, num2: u32) -> ZomeApiResult<u32> {
    Ok(num1 + num2)
}

define_zome! {
    entries: []

    init: || {
        Ok(())
    }

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {
        Ok(())
    }

    functions: [
        sum: {
            inputs: |num1: u32, num2: u32|,
            outputs: |sum: ZomeApiResult<u32>|,
            handler: handle_sum
        }
    ]

    traits: {
        hc_public [sum]

        // This is to make sure the test for introspection/trait/get_zome_by_trait
        // not only checks the trait name but also the function signatures in the trait.
        // The according test expects to NOT get this zome as result for the crypto trait
        // that is appropriately implemented only in the simple zome.
        crypto [sum]
    }

}

#[cfg(test)]
mod tests {

    use super::handle_sum;

    #[test]
    pub fn handle_sum_test() {
        assert_eq!(
            handle_sum(1, 1),
            Ok(2),
        );
    }

}
