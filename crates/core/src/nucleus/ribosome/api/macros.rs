#[macro_export]
macro_rules! link_zome_api {
    (
        $(
            $(#[$meta:meta])*
            $internal_name:literal, $enum_variant:ident, $function_name:path ;
        )*
    ) => {

        /// Enumeration of all the Zome Functions known and usable in Zomes.
        /// Enumeration can convert to str.
        #[repr(usize)]
        #[derive(FromPrimitive, Debug, PartialEq, Eq)]
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

        /// does nothing, escape hatch so the compiler can enforce exhaustive matching in as_fn
        fn noop(_runtime: &mut Runtime, _args: &RuntimeArgs) -> ZomeApiResult {
            ribosome_success!()
        }

        impl ZomeApiFunction {
            // cannot test this because PartialEq is not implemented for fns
            #[cfg_attr(tarpaulin, skip)]
            pub fn as_fn(&self) -> impl Fn(&mut Runtime, &RuntimeArgs) -> ZomeApiResult {
                // TODO Implement a proper "abort" function for handling assemblyscript aborts
                // @see: https://github.com/holochain/holochain-rust/issues/324

                match *self {
                    ZomeApiFunction::MissingNo => noop,
                    ZomeApiFunction::Abort => noop,
                    $( ZomeApiFunction::$enum_variant => $function_name , )*
                }
            }
        }
    };
}
