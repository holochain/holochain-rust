extern crate proc_macro2;

use crate::zome_code_def::{
    EntryDefCallbacks, FnDeclaration, FnParameter, InitCallback, ReceiveCallback,
    ValidateAgentCallback, ZomeCodeDef, ZomeFunction, ZomeFunctions,
};

use hdk::holochain_core_types::dna::{fn_declarations::TraitFns, zome::ZomeTraits};
use std::collections::BTreeMap;

static INIT_ATTRIBUTE: &str = "init";
static VALIDATE_AGENT_ATTRIBUTE: &str = "validate_agent";
static ZOME_FN_ATTRIBUTE: &str = "zome_fn";
static ENTRY_DEF_ATTRIBUTE: &str = "entry_def";
static RECEIVE_CALLBACK_ATTRIBUTE: &str = "receive";

pub trait IntoZome {
    fn extract_zome_fns(&self) -> ZomeFunctions;
    fn extract_entry_defs(&self) -> EntryDefCallbacks;
    fn extract_init(&self) -> InitCallback;
    fn extract_validate_agent(&self) -> ValidateAgentCallback;
    fn extract_traits(&self) -> ZomeTraits;
    fn extract_receive_callback(&self) -> Option<ReceiveCallback>;
    fn extract_extra(&self) -> Vec<syn::Item>;

    fn extract_zome(&self) -> ZomeCodeDef {
        ZomeCodeDef {
            traits: self.extract_traits(),
            entry_def_fns: self.extract_entry_defs(),
            init: self.extract_init(),
            validate_agent: self.extract_validate_agent(),
            receive_callback: self.extract_receive_callback(),
            zome_fns: self.extract_zome_fns(),
            extra: self.extract_extra(),
        }
    }
}

//////////////////////////////////////////////////////////////////////////

fn emit_error(target: &proc_macro2::Ident, message: &str) {
    target.span().unstable().error(message).emit();
}

fn emit_warning(target: &proc_macro2::Ident, message: &str) {
    target.span().unstable().warning(message).emit();
}

// Return an iterator over the syn::ItemFn in a module
fn funcs_iter(module: &syn::ItemMod) -> impl Iterator<Item = syn::ItemFn> {
    module
        .clone()
        .content
        .unwrap()
        .1
        .into_iter()
        .filter_map(|item| match item {
            syn::Item::Fn(func) => Some(func),
            _ => None,
        })
}

fn is_tagged_with(tag: &'static str) -> impl Fn(&syn::ItemFn) -> bool {
    move |func| func.attrs.iter().any(|attr| attr.path.is_ident(tag))
}

fn zome_fn_dec_from_syn(func: &syn::ItemFn) -> FnDeclaration {
    let inputs = func
        .decl
        .inputs
        .iter()
        .map(|e| {
            if let syn::FnArg::Captured(arg) = e {
                let ident = match &arg.pat {
                    syn::Pat::Ident(name_ident) => name_ident.ident.clone(),
                    _ => {
                        emit_error(
                            &func.ident,
                            "not a valid parameter pattern for zome function",
                        );
                        panic!()
                    }
                };
                let ty = match arg.ty.clone() {
                    syn::Type::Path(type_path) => type_path,
                    _ => {
                        emit_error(&func.ident, "Invalid type for zome function parameter");
                        panic!()
                    }
                };
                FnParameter { ident, ty }
            } else {
                emit_error(&func.ident, "could not parse function args");
                panic!()
            }
        })
        .collect();

    FnDeclaration {
        name: func.ident.clone().to_string(),
        inputs,
        output: func.decl.output.clone(),
    }
}

impl IntoZome for syn::ItemMod {
    fn extract_init(&self) -> InitCallback {
        // find all the functions tagged as the init callback
        let geneses: Vec<Box<syn::Block>> = funcs_iter(self)
            .filter(is_tagged_with(INIT_ATTRIBUTE))
            .fold(Vec::new(), |mut acc, func| {
                acc.push(func.block);
                acc
            });
        // only a single function can be tagged as init in a valid Zome.
        // Error if there is more than one
        // Also error if there is no init
        match geneses.len() {
            0 => {
                emit_error(&self.ident,
                    "No init function defined! A zome definition requires a callback tagged with #[init]");
                panic!()
            }
            1 => *geneses[0].clone(),
            _ => {
                emit_error(&self.ident,
                    "Multiple functions tagged as init callback! Only one is permitted per zome definition.");
                panic!()
            }
        }
    }

    fn extract_validate_agent(&self) -> ValidateAgentCallback {
        // find all the functions tagged as the validate_agent callback
        let callbacks: Vec<syn::ItemFn> = funcs_iter(self)
            .filter(is_tagged_with(VALIDATE_AGENT_ATTRIBUTE))
            .fold(Vec::new(), |mut acc, func| {
                acc.push(func);
                acc
            });
        // only a single function can be tagged as validate_agent in a valid Zome.
        // Error if there is more than one
        // Also error if tagged function
        match callbacks.len() {
            0 => {
                emit_error(&self.ident,
                    "No validate_agent function defined! A zome definition requires a callback tagged with #[validate_agent]");
                panic!()
            }
            1 => {
                let callback = callbacks[0].clone();
                let fn_def = zome_fn_dec_from_syn(&callback);

                // must have the valid function signature which is ($ident: EntryValidationData<AgentId>)
                let validation_data_param = match fn_def.inputs.len() {
                    1 => {
                        let param = fn_def.inputs[0].clone();
                        param.ident
                    }
                    _ => {
                        emit_error(&self.ident,
                            "incorrect number of params for validate_agent callback. Must have a single param with type `EntryValidationData<AgentId>`");
                        panic!()
                    }
                };

                ValidateAgentCallback {
                    validation_data_param,
                    code: (*callback.block),
                }
            }
            _ => {
                emit_error(&self.ident,
                    "Multiple functions tagged as validate_agent callback! Only one is permitted per zome definition.");
                panic!()
            }
        }
    }

    fn extract_zome_fns(&self) -> ZomeFunctions {
        // find all the functions tagged as the zome_fn
        funcs_iter(self)
            .filter(is_tagged_with(ZOME_FN_ATTRIBUTE))
            .fold(Vec::new(), |mut acc, func| {
                let fn_def = zome_fn_dec_from_syn(&func);
                acc.push(ZomeFunction {
                    declaration: fn_def,
                    code: *func.block,
                });
                acc
            })
    }

    fn extract_entry_defs(&self) -> Vec<syn::ItemFn> {
        funcs_iter(self)
            .filter(is_tagged_with(ENTRY_DEF_ATTRIBUTE))
            .fold(Vec::new(), |mut acc, mut func| {
                // Drop the EntryDef attribute on the functions so this doesn't recurse
                func.attrs = func
                    .attrs
                    .into_iter()
                    .filter(|attr| !attr.path.is_ident(ENTRY_DEF_ATTRIBUTE))
                    .collect();
                acc.push(func);
                acc
            })
    }

    fn extract_traits(&self) -> ZomeTraits {
        funcs_iter(self)
	    .filter(is_tagged_with(ZOME_FN_ATTRIBUTE))
	    .fold(BTreeMap::new(), |mut acc, func| {
            let func_name = func.ident.to_string();
            func.attrs
            .iter()
            .filter(|attr| attr.path.is_ident(ZOME_FN_ATTRIBUTE))
            .for_each(|attr| {
                let meta = attr.parse_meta().unwrap();
                match meta {
                	syn::Meta::List(meta_list) => {
		                meta_list.nested.iter().for_each(|e| {
		                    if let syn::NestedMeta::Literal(syn::Lit::Str(lit)) = e {
		                        let trait_name = lit.value().clone();
		                        if acc.get(&trait_name).is_none() {
		                            acc.insert(trait_name.clone(), TraitFns::new());
		                        }
		                        acc.get_mut(&trait_name).unwrap().functions.push(func_name.clone());
		                    }
		                });
                	},
                	syn::Meta::Word(_) => emit_warning(&func.ident,
                        "Function is tagged as zome_fn but is not exposed via a Holochain trait. Did you mean to expose it publicly '#[zome_fn(\"hc_public\")]'?"),
                	_ => emit_error(&func.ident,
                        "zome_fn must be preceded by a comma delimited list of Holochain traits e.g. #[zome_fn(\"hc_public\", \"custom_trait\")"),
                }
            });
	        acc
	    })
    }

    // For this implementation the `extra` is all the content of the module that is not tagged as special
    // Without this the author can't write custom structs in the module
    fn extract_extra(&self) -> Vec<syn::Item> {
        match self.content.clone() {
            Some((_, items)) => {
                items
                    .into_iter()
                    .filter(|item| {
                        if let syn::Item::Fn(func) = item {
                            // any functions not tagged with a hdk attribute
                            !is_tagged_with(ZOME_FN_ATTRIBUTE)(func)
                                && !is_tagged_with(INIT_ATTRIBUTE)(func)
                                && !is_tagged_with(ENTRY_DEF_ATTRIBUTE)(func)
                                && !is_tagged_with(RECEIVE_CALLBACK_ATTRIBUTE)(func)
                                && !is_tagged_with(VALIDATE_AGENT_ATTRIBUTE)(func)
                        } else {
                            true // and anything that is not a function
                        }
                    })
                    .collect()
            }
            None => Vec::new(),
        }
    }

    fn extract_receive_callback(&self) -> Option<ReceiveCallback> {
        // find all the functions tagged as the receive callback
        let callbacks: Vec<ReceiveCallback> = funcs_iter(self)
            .filter(is_tagged_with(RECEIVE_CALLBACK_ATTRIBUTE))
            .fold(Vec::new(), |mut acc, func| {
                let inputs = func.decl.inputs;

                match inputs.len() {
                    2 => {
                        let params = inputs.iter().take(2).collect::<Vec<_>>();

                        match (params[0], params[1]) {
                            (
                                syn::FnArg::Captured(syn::ArgCaptured{pat: syn::Pat::Ident(from_ident), ..}),
                                syn::FnArg::Captured(syn::ArgCaptured{pat: syn::Pat::Ident(message_ident), ..})
                            ) => {
                                acc.push(ReceiveCallback {
                                    from_param: from_ident.ident.clone(),
                                    message_param: message_ident.ident.clone(),
                                    code: *func.block,
                                });
                            },
                            _ => {
                                emit_error(
                                    &func.ident,
                                    "Receive callback must take two named arguments of type 'Address' and 'String' respectively",
                                );
                                panic!()
                            }
                        }
                    }
                    _ => {
                        emit_error(
                            &func.ident,
                            "Receive callback must take two named arguments of type 'Address' and 'String' respectively",
                        );
                        panic!()
                    }
                }
                acc
            });
        match callbacks.len() {
            0 => None,
            1 => Some(callbacks[0].clone()),
            _ => {
                emit_error(
                    &self.ident,
                    "Multiple functions tagged with receive. Only one permitted per zome.",
                );
                panic!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_init_smoke_test() {
        let module: syn::ItemMod = parse_quote! {
            mod zome {
                #[init]
                fn init() {
                    Ok(())
                }

                #[validate_agent]
                fn validate_agent(validation_data: EntryValidationData<AgentId>) {
                    Ok(())
                }
            }
        };
        let _ = module.extract_zome();
    }

    #[test]
    fn test_extract_single_trait() {
        let module: syn::ItemMod = parse_quote! {
            mod zome {
                #[init]
                fn init() {
                    Ok(())
                }

                #[validate_agent]
                fn validate_agent(validation_data: EntryValidationData<AgentId>) {
                    Ok(())
                }

                #[zome_fn("trait_name")]
                fn a_fn() {
                    Ok(())
                }
            }
        };
        let zome_def = module.extract_zome();
        let mut expected_traits: ZomeTraits = BTreeMap::new();
        expected_traits.insert(
            "trait_name".to_string(),
            TraitFns {
                functions: vec!["a_fn".to_string()],
            },
        );
        assert_eq! {
            zome_def.traits,
            expected_traits
        }
    }

    #[test]
    fn test_multi_function_multi_traits() {
        let module: syn::ItemMod = parse_quote! {
            mod zome {
                #[init]
                fn init() {
                    Ok(())
                }

                #[validate_agent]
                fn validate_agent(validation_data: EntryValidationData<AgentId>) {
                    Ok(())
                }

                #[zome_fn("trait1", "trait2")]
                fn a_fn() {
                    Ok(())
                }

                #[zome_fn("trait2", "trait3")]
                fn b_fn() {
                    Ok(())
                }
            }
        };
        let zome_def = module.extract_zome();
        let mut expected_traits: ZomeTraits = BTreeMap::new();
        expected_traits.insert(
            "trait1".to_string(),
            TraitFns {
                functions: vec!["a_fn".to_string()],
            },
        );
        expected_traits.insert(
            "trait2".to_string(),
            TraitFns {
                functions: vec!["a_fn".to_string(), "b_fn".to_string()],
            },
        );
        expected_traits.insert(
            "trait3".to_string(),
            TraitFns {
                functions: vec!["b_fn".to_string()],
            },
        );

        assert_eq! {
            zome_def.traits,
            expected_traits
        }
    }

    #[test]
    fn test_extract_function_params_and_return() {
        let module: syn::ItemMod = parse_quote! {
            mod zome {
                #[init]
                fn init() {
                    Ok(())
                }

                #[validate_agent]
                fn validate_agent(validation_data: EntryValidationData<AgentId>) {
                    Ok(())
                }

                #[zome_fn("test_trait")]
                fn a_fn(param1: i32, param2: String, param3: bool) -> String {
                    "test".into()
                }
            }
        };
        let zome_def = module.extract_zome();

        assert_eq! {
            zome_def.zome_fns.first().unwrap().declaration,
            FnDeclaration{
                name: "a_fn".to_string(),
                inputs: vec![
                    FnParameter::new_from_str("param1", "i32"),
                      FnParameter::new_from_str("param2", "String"),
                    FnParameter::new_from_str("param3", "bool"),
                ],
                output: syn::parse_quote!(-> String)
            }
        }
    }

    #[test]
    fn test_extract_function_with_generic_return() {
        let module: syn::ItemMod = parse_quote! {
            mod zome {
                #[init]
                fn init() {
                    Ok(())
                }

                #[validate_agent]
                fn validate_agent(validation_data: EntryValidationData<AgentId>) {
                    Ok(())
                }

                #[zome_fn("hc_public")]
                fn a_fn() -> ZomeApiResult<String> {
                    Ok("test".into())
                }
            }
        };
        let zome_def = module.extract_zome();

        assert_eq! {
            zome_def.zome_fns.first().unwrap().declaration,
            FnDeclaration{
                name: "a_fn".to_string(),
                inputs: vec![],
                output: syn::parse_quote!(-> ZomeApiResult<String>),
            }
        }
    }

    #[test]
    fn test_single_entry() {
        let module: syn::ItemMod = parse_quote! {
            mod zome {
                #[init]
                fn init() {
                    Ok(())
                }

                #[validate_agent]
                fn validate_agent(validation_data: EntryValidationData<AgentId>) {
                    Ok(())
                }

                #[entry_def]
                fn test_entry_def() {
                    entry!(
                        name: "testEntryType",
                        description: "asdfda",
                        sharing: Sharing::Public,
                        validation_package: || {
                            hdk::ValidationPackageDefinition::ChainFull
                        },
                        validation: |_validation_data: hdk::EntryValidationData<TestEntryType>| {
                            Ok(())
                        }
                    )
                }
            }
        };
        let zome_def = module.extract_zome();
        assert_eq! {
            zome_def.entry_def_fns.len(),
            1
        }
    }

    #[test]
    fn test_extra_code_in_module() {
        let module: syn::ItemMod = parse_quote! {
            mod zome {
                #[init]
                fn init() {
                    Ok(())
                }

                #[validate_agent]
                fn validate_agent(validation_data: EntryValidationData<AgentId>) {
                    Ok(())
                }

                 const SOME_CONST: u32 = 123;

                fn non_zome_func() {
                    Ok(())
                }

                struct SomeOtherStruct {
                    field: String
                }
            }
        };
        let zome_def = module.extract_zome();

        assert_eq! {
            zome_def.extra.len(),
            3
        }
    }

    #[test]
    fn test_no_receive_callback() {
        let module: syn::ItemMod = parse_quote! {
            mod zome {
                #[init]
                fn init() {
                    Ok(())
                }

                #[validate_agent]
                fn validate_agent(validation_data: EntryValidationData<AgentId>) {
                    Ok(())
                }
            }
        };
        let zome_def = module.extract_zome();
        assert!(zome_def.receive_callback.is_none())
    }

    #[test]
    fn test_receive_callback() {
        let module: syn::ItemMod = parse_quote! {
            mod zome {
                #[init]
                fn init() {
                    Ok(())
                }

                #[validate_agent]
                fn validate_agent(validation_data: EntryValidationData<AgentId>) {
                    Ok(())
                }

                #[receive]
                fn receive(from :Address, message: String) {
                    Ok(())
                }
            }
        };
        let zome_def = module.extract_zome();
        assert!(zome_def.receive_callback.is_some())
    }
}
