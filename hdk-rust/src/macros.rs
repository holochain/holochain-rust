#[macro_export]
macro_rules! load_json {
    ($encoded_allocation_of_input:ident) => {{
        let maybe_input =
            $crate::holochain_wasm_utils::memory_serialization::load_json($encoded_allocation_of_input);
        if maybe_input.is_err() {
            return $crate::holochain_wasm_utils::holochain_core_types::error::RibosomeErrorCode::ArgumentDeserializationFailed
                as u32;
        }
        maybe_input
    }};
}

/// A macro for describing zomes
#[macro_export]
macro_rules! define_zome {
    (
        entries : [
            $( $entry_expr:expr ),*
        ]

        genesis : || {
            $genesis_expr:expr
        }

        functions : {
            $(
                $cap:ident ( $vis:ident ) {
                    $(
                        $zome_function_name:ident : {
                            inputs: | $( $input_param_name:ident : $input_param_type:ty ),* |,
                            outputs: | $( $output_param_name:ident : $output_param_type:ty ),* |,
                            handler: $handler_path:path
                        }
                    )+
                }
            )*
        }

    ) => {
        #[no_mangle]
        #[allow(unused_variables)]
        pub extern "C" fn zome_setup(zd: &mut $crate::meta::ZomeDefinition) {
            $(
                zd.define($entry_expr);
            )*
        }

        #[no_mangle]
        pub extern "C" fn genesis(encoded_allocation_of_input: u32) -> u32 {
            $crate::global_fns::init_global_memory(encoded_allocation_of_input);

            fn execute() -> Result<(), String> {
                $genesis_expr
            }

            $crate::global_fns::store_and_return_output(execute())
        }

        use $crate::holochain_dna::zome::capabilities::Capability;
        use std::collections::HashMap;

        #[no_mangle]
        #[allow(unused_imports)]
        pub fn __list_capabilities() -> HashMap<String, Capability> {

            use $crate::holochain_dna::zome::capabilities::{Capability, Membrane, CapabilityType, FnParameter, FnDeclaration};
            use std::collections::HashMap;

            let return_value: HashMap<String, Capability> = {
                let mut cap_map = HashMap::new();

                $(
                    {
                        let mut capability = Capability::new();
                        capability.cap_type = CapabilityType { membrane: Membrane::$vis };
                        capability.functions = vec![
                            $(
                                FnDeclaration {
                                    name: stringify!($zome_function_name).into(),
                                    inputs: vec![
                                        $(
                                            FnParameter::new(stringify!($input_param_name), stringify!($input_param_type))
                                        ),*
                                    ],
                                    outputs: vec![
                                        $(
                                            FnParameter::new(stringify!($output_param_name), stringify!($output_param_type))
                                        ),*
                                    ]
                                }

                            ),+
                        ];

                        cap_map.insert(stringify!($cap).into(), capability);
                    }
                ),*

                cap_map
            };

            return_value
        }

        $(
            $(
                #[no_mangle]
                pub extern "C" fn $zome_function_name(encoded_allocation_of_input: u32) -> u32 {
                    $crate::global_fns::init_global_memory(encoded_allocation_of_input);

                    // Macro'd InputStruct
                    #[derive(Deserialize)]
                    struct InputStruct {
                        $($input_param_name : $input_param_type),*
                    }

                    // #[derive(Serialize)]
                    // struct OutputStruct {
                    //     $( $output_param_name:ident : $output_param_type:ty ),*
                    // }

                    // Deserialize input
                    let maybe_input = load_json!(encoded_allocation_of_input);
                    let input: InputStruct = maybe_input.unwrap();

                    // Macro'd function body
                    fn execute(params: InputStruct) -> impl ::serde::Serialize {
                        let InputStruct { $($input_param_name),* } = params;

                        $handler_path($($input_param_name),*)
                    }

                    // Execute inner function
                    let output_obj = execute(input);

                    $crate::global_fns::store_and_return_output(output_obj)
                }
            )+
        )*
    };
}
