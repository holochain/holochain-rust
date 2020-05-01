#[macro_export]
macro_rules! link_zome_api {
    (
        $(
            $(#[$meta:meta])*
            $internal_name:literal, $enum_variant:ident, $function_name:path ;
        )*
    ) => {

        use crate::nucleus::{
            actions::{trace_invoke_hdk_function::trace_invoke_hdk_function, trace_return_hdk_function::trace_return_hdk_function},
            HdkFnCall,
        };
        use crate::wasm_engine::runtime::WasmCallData;
        use holochain_json_api::json::JsonString;

        /// Enumeration of all the Zome Functions known and usable in Zomes.
        /// Enumeration can convert to str.
        #[repr(usize)]
        #[derive(FromPrimitive, Clone, Hash, Debug, PartialEq, Eq, Serialize)]
        pub enum ZomeApiFunction {
            /// Error index for unimplemented functions
            MissingNo = 0,

            /// Abort is a way to receive useful debug info from
            /// assemblyscript memory allocators
            /// message: mem address in the wasm memory for an error message
            /// filename: mem address in the wasm memory for a filename
            /// line: line number
            /// column: column number
            Abort,

            $(
                $(#[$meta])*
                $enum_variant
            ),*
        }

        impl Defn for ZomeApiFunction {
            fn as_str(&self) -> &'static str {
                match *self {
                    ZomeApiFunction::MissingNo => "",
                    ZomeApiFunction::Abort => "abort",
                    $(ZomeApiFunction::$enum_variant => $internal_name),*
                }
            }

            fn str_to_index(s: &str) -> usize {
                ZomeApiFunction::from_str(s).unwrap_or(ZomeApiFunction::MissingNo) as usize
            }

            fn from_index(i: usize) -> Self {
                FromPrimitive::from_usize(i).unwrap_or(ZomeApiFunction::MissingNo)
            }
        }

        impl FromStr for ZomeApiFunction {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "abort" => Ok(ZomeApiFunction::Abort),
                    $($internal_name => Ok(ZomeApiFunction::$enum_variant),)*
                    _ => Err("Cannot convert string to ZomeApiFunction"),
                }
            }
        }

        impl ZomeApiFunction {
            // cannot test this because PartialEq is not implemented for fns
            #[cfg_attr(tarpaulin, skip)]
            pub fn apply(&self, runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
                // TODO Implement a proper "abort" function for handling assemblyscript aborts
                // @see: https://github.com/holochain/holochain-rust/issues/324

                match *self {
                    ZomeApiFunction::MissingNo => ribosome_success!(),
                    ZomeApiFunction::Abort => ribosome_success!(),
                    $( ZomeApiFunction::$enum_variant => {
                        if let Ok(context) = runtime.context() {
                            if let WasmCallData::ZomeCall(zome_call_data) = runtime.data.clone() {
                                let zome_api_call = zome_call_data.call;
                                let parameters = runtime.load_json_string_from_args(&args);
                                let hdk_fn_call = HdkFnCall { function: self.clone(), parameters };
                                trace_invoke_hdk_function(zome_api_call.clone(), hdk_fn_call.clone(), &context);
                                let result = $function_name(runtime, args);
                                let hdk_fn_result = Ok(JsonString::from("TODO"));
                                trace_return_hdk_function(zome_api_call.clone(), hdk_fn_call, hdk_fn_result, &context);
                                result
                            } else {
                                // we don't record hdk function calls for callbacks or direct function calls
                                $function_name(runtime, args)
                            }
                        } else {
                            error!("Could not get context for runtime");
                            $function_name(runtime, args)
                        }
                    } , )*
                }
            }
        }
    };
}
