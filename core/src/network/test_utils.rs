use crate::{
    context::Context,
    instance::{tests::test_context, Instance},
};
use holochain_core_types::{cas::content::Address, dna::Dna};
use std::sync::Arc;

/// create a test instance
#[cfg_attr(tarpaulin, skip)]
pub fn test_instance_with_spoofed_dna(
    dna: Dna,
    spoofed_dna_address: Address,
    name: &str,
) -> Result<(Instance, Arc<Context>), String> {
    // Create instance and plug in our DNA
    let context = test_context(name, None);
    let mut instance = Instance::new(context.clone());
    let context = instance.initialize_with_spoofed_dna(dna.clone(), spoofed_dna_address, context)?;

    assert_eq!(instance.state().nucleus().dna(), Some(dna.clone()));
    assert!(instance.state().nucleus().has_initialized());

    /// fair warning... use test_instance_blank() if you want a minimal instance
    assert!(
        !dna.zomes.clone().is_empty(),
        "Empty zomes = No genesis = infinite loops below!"
    );

    Ok((instance, context))
}

pub fn test_wat_always_valid() -> String {
    r#"
(module

    (memory 1)
    (export "memory" (memory 0))

    (func
        (export "__hdk_validate_app_entry")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
    )

    (func
        (export "__hdk_validate_link")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
    )


    (func
        (export "__hdk_get_validation_package_for_entry_type")
        (param $allocation i64)
        (result i64)

        ;; This writes "Entry" into memory
        (i64.store (i32.const 0) (i64.const 34))
        (i64.store (i32.const 1) (i64.const 69))
        (i64.store (i32.const 2) (i64.const 110))
        (i64.store (i32.const 3) (i64.const 116))
        (i64.store (i32.const 4) (i64.const 114))
        (i64.store (i32.const 5) (i64.const 121))
        (i64.store (i32.const 6) (i64.const 34))

        (i64.const 7)
    )

    (func
        (export "__hdk_get_validation_package_for_link")
        (param $allocation i64)
        (result i64)

        ;; This writes "Entry" into memory
        (i64.store (i32.const 0) (i64.const 34))
        (i64.store (i32.const 1) (i64.const 69))
        (i64.store (i32.const 2) (i64.const 110))
        (i64.store (i32.const 3) (i64.const 116))
        (i64.store (i32.const 4) (i64.const 114))
        (i64.store (i32.const 5) (i64.const 121))
        (i64.store (i32.const 6) (i64.const 34))

        (i64.const 7)
    )

    (func
        (export "__list_capabilities")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
    )

    (func
        (export "__list_functions")
        (param $allocation i32)
        (result i32)

        (i32.const 0)
    )
)
                "#
    .to_string()
}

pub fn test_wat_always_invalid() -> String {
    r#"
(module

    (memory 1)
    (export "memory" (memory 0))

    (func
        (export "__hdk_validate_app_entry")
        (param $allocation i64)
        (result i64)

        ;; This writes "FAIL wat" into memory
        (i64.store (i32.const 0) (i64.const 70))
        (i64.store (i32.const 1) (i64.const 65))
        (i64.store (i32.const 2) (i64.const 73))
        (i64.store (i32.const 3) (i64.const 76))
        (i64.store (i32.const 4) (i64.const 32))
        (i64.store (i32.const 5) (i64.const 119))
        (i64.store (i32.const 6) (i64.const 97))
        (i64.store (i32.const 7) (i64.const 116))

        (i64.const 8)
    )

    (func
        (export "__hdk_validate_link")
        (param $allocation i64)
        (result i64)

        ;; This writes "FAIL wat" into memory
        (i64.store (i32.const 0) (i64.const 70))
        (i64.store (i32.const 1) (i64.const 65))
        (i64.store (i32.const 2) (i64.const 73))
        (i64.store (i32.const 3) (i64.const 76))
        (i64.store (i32.const 4) (i64.const 32))
        (i64.store (i32.const 5) (i64.const 119))
        (i64.store (i32.const 6) (i64.const 97))
        (i64.store (i32.const 7) (i64.const 116))

        (i64.const 8)
    )


    (func
        (export "__hdk_get_validation_package_for_entry_type")
        (param $allocation i64)
        (result i64)

        ;; This writes "Entry" into memory
        (i64.store (i32.const 0) (i64.const 34))
        (i64.store (i32.const 1) (i64.const 69))
        (i64.store (i32.const 2) (i64.const 110))
        (i64.store (i32.const 3) (i64.const 116))
        (i64.store (i32.const 4) (i64.const 114))
        (i64.store (i32.const 5) (i64.const 121))
        (i64.store (i32.const 6) (i64.const 34))

        (i64.const 7)
    )

    (func
        (export "__hdk_get_validation_package_for_link")
        (param $allocation i64)
        (result i64)

        ;; This writes "Entry" into memory
        (i64.store (i32.const 0) (i64.const 34))
        (i64.store (i32.const 1) (i64.const 69))
        (i64.store (i32.const 2) (i64.const 110))
        (i64.store (i32.const 3) (i64.const 116))
        (i64.store (i32.const 4) (i64.const 114))
        (i64.store (i32.const 5) (i64.const 121))
        (i64.store (i32.const 6) (i64.const 34))

        (i64.const 7)
    )

    (func
        (export "__list_capabilities")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
    )

    (func
        (export "__list_functions")
        (param $allocation i32)
        (result i32)

        (i32.const 0)
    )
)
                "#
    .to_string()
}
