extern crate proc_macro2;

use std::collections::BTreeMap;
use hdk::holochain_core_types::dna::{
    zome::{ZomeTraits},
    fn_declarations::{FnDeclaration, FnParameter, TraitFns},
};

static GENESIS_ATTRIBUTE: &str = "genesis";
static ZOME_FN_ATTRIBUTE: &str = "zome_fn";
static ENTRY_DEF_ATTRIBUTE: &str = "entry_def";
static RECEIVE_CALLBACK_ATTRIBUTE: &str = "receive";

pub type GenesisCallback = syn::Block;
pub type ZomeFunctionCode = syn::Block;
pub type EntryDefCallback = syn::ItemFn;
#[derive(Clone)]
pub struct ReceiveCallback {
    pub param: syn::Ident,
    pub code: syn::Block,
}

#[derive(Clone)]
pub struct ZomeFunction {
    pub declaration: FnDeclaration,
    pub code: ZomeFunctionCode,
}

pub type ZomeFunctions = Vec<ZomeFunction>;
pub type EntryDefCallbacks = Vec<EntryDefCallback>;

pub struct ZomeCodeDef {
    pub genesis: GenesisCallback,
    pub zome_fns: ZomeFunctions, // receive: ReceiveCallbacks
    pub entry_def_fns: Vec<syn::ItemFn>,
    pub traits: ZomeTraits,
    pub receive_callback: Option<ReceiveCallback>,
    pub extra: Vec<syn::Item>, // extra stuff to be added as is to the zome code
}

pub trait IntoZome {
	fn extract_zome_fns(&self) -> ZomeFunctions;
	fn extract_entry_defs(&self) -> EntryDefCallbacks;
	fn extract_genesis(&self) -> GenesisCallback;
	fn extract_traits(&self) -> ZomeTraits;
    fn extract_receive_callback(&self) -> Option<ReceiveCallback>;
	fn extract_extra(&self) -> Vec<syn::Item>;

	fn extract_zome(&self) -> ZomeCodeDef {
		ZomeCodeDef {
            traits: self.extract_traits(),
            entry_def_fns: self.extract_entry_defs(),
            genesis: self.extract_genesis(),
            receive_callback: self.extract_receive_callback(),
            zome_fns: self.extract_zome_fns(),
            extra: self.extract_extra(),
        }
	}
}


//////////////////////////////////////////////////////////////////////////

// Return an iterator over the syn::ItemFn in a module
fn funcs_iter(module: &syn::ItemMod) -> impl Iterator<Item = syn::ItemFn> {
	module
    .clone()
    .content
    .unwrap()
    .1
    .into_iter()
    .filter_map(|item| {
    	match item {
    		syn::Item::Fn(func) => Some(func),
    		_ => None,
    	}
    })
}

fn is_tagged_with(tag: &'static str) -> impl Fn(&syn::ItemFn) -> bool {
	move |func| {
		func.attrs.iter().any(|attr| attr.path.is_ident(tag))
	}
}

fn zome_fn_dec_from_syn(func: &syn::ItemFn) -> FnDeclaration {
    let inputs = func
        .decl
        .inputs
        .iter()
        .map(|e| {
            if let syn::FnArg::Captured(arg) = e {
                let name: String = match &arg.pat {
                    syn::Pat::Ident(name_ident) => name_ident.ident.to_string(),
                    _ => "".into(),
                };
                let parameter_type: String = match &arg.ty {
                    syn::Type::Path(type_path) => type_path
                        .path
                        .segments
                        .iter()
                        .map(|segment| {
                            segment.ident.to_string()
                        })
                        .collect(),
                    _ => "".into(),
                };
                FnParameter {
                    name,
                    parameter_type,
                }
            } else {
                panic!("could not parse function args")
            }
        })
        .collect();

    let output_type: String = match &func.decl.output {
        syn::ReturnType::Default => "()".to_string(),
        syn::ReturnType::Type(_, ty) => match *(*ty).clone() {
            syn::Type::Path(type_path) => type_path
                .path
                .segments
                .iter()
                .next()
                .unwrap()
                .ident
                .to_string(),
            _ => "".into(),
        },
    };

    FnDeclaration {
        name: func.ident.clone().to_string(),
        inputs: inputs,
        outputs: vec![FnParameter::new("result", &output_type)],
    }
}

impl IntoZome for syn::ItemMod {

	fn extract_genesis(&self) -> GenesisCallback {
	    // find all the functions tagged as the genesis callback
	    let geneses: Vec<Box<syn::Block>> =
        funcs_iter(self)
        .filter(is_tagged_with(GENESIS_ATTRIBUTE))
        .fold(Vec::new(), |mut acc, func| {
            acc.push(func.block);
            acc
        });
	    // only a single function can be tagged in a valid some so error if there is more than one
	    // if there is None then use the sensible default of Ok(())
	    match geneses.len() {
	        0 => {
	            self.ident.span().unstable()
	            .error("No genesis function defined! A zome definition requires a callback tagged with #[genesis]")
	            .emit();
	            panic!()
	        }
	        1 => *geneses[0].clone(),
	        _ => {
	            self.ident.span().unstable()
	            .error("Multiple functions tagged as genesis callback! Only one is permitted per zome definition.")
	            .emit();
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
		    // drop all attributes on the fn. This may cause problems
		    // and really should only drop the ENTRY_DEF_ATTRIBUTE
		    func.attrs = Vec::new();
		    acc.push(func);
	        acc
	    })
	}

	fn extract_traits(&self) -> ZomeTraits {
	    funcs_iter(self)
	    .filter(is_tagged_with(ZOME_FN_ATTRIBUTE))
	    .fold(BTreeMap::new(), |mut acc, func| {
            let func_name = func.ident.to_string();
            func.attrs.iter().for_each(|attr| { // this will error if zome fn has multiple attriutes defined

                let meta = attr.parse_meta().unwrap();
                match meta {
                	syn::Meta::List(meta_list) => {
		                meta_list.nested.iter().for_each(|e| {
		                    if let syn::NestedMeta::Literal(syn::Lit::Str(lit)) = e {
		                        let trait_name = lit.value().clone();
		                        if let None = acc.get(&trait_name) {
		                            acc.insert(trait_name.clone(), TraitFns::new());
		                        }
		                        acc.get_mut(&trait_name).unwrap().functions.push(func_name.clone());
		                    }
		                });
                	},
                	syn::Meta::Word(_) => func.ident.span().unstable().warning("Function is tagged as zome_fn but is not exposed via a trait. Did you mean to expose it publicly '#[zome_fn(\"hc_public\")]'?").emit(),
                	_ => func.ident.span().unstable().error("zome_fn must be preceded by a comma delimited list of traits e.g. #[zome_fn(\"hc_public\", \"custom_trait\")").emit(),
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
						!is_tagged_with(ZOME_FN_ATTRIBUTE)(func) &&
						!is_tagged_with(GENESIS_ATTRIBUTE)(func) &&
						!is_tagged_with(ENTRY_DEF_ATTRIBUTE)(func) &&
                        !is_tagged_with(RECEIVE_CALLBACK_ATTRIBUTE)(func)
					} else {
						true // and anything that is not a function
					}
				})
				.collect()
			},
			None => Vec::new(),
		}
	}

    fn extract_receive_callback(&self) -> Option<ReceiveCallback> {
        // find all the functions tagged as the genesis callback
        let callbacks: Vec<ReceiveCallback> =
        funcs_iter(self)
        .filter(is_tagged_with(RECEIVE_CALLBACK_ATTRIBUTE))
        .fold(Vec::new(), |mut acc, func| {
            let inputs = func.decl.inputs;

            match inputs.len() {
                1 => {
                    let input = inputs.iter().next().unwrap();
                    if let syn::FnArg::Captured(arg) = input {
                        let name = match &arg.pat {
                            syn::Pat::Ident(name_ident) => name_ident.ident.clone(),
                            _ => { 
                                func.ident.span().unstable()
                                .error("The argument to receive must have a name")
                                .emit();
                                panic!()
                            },
                        };
                        acc.push(ReceiveCallback{
                            param: name,
                            code: *func.block,
                        });
                    }

                },
                _ => { 
                    func.ident.span().unstable()
                    .error("Receive callback must take a single argument of type 'String'")
                    .emit();
                    panic!()
                },
            }
            acc
        });
        match callbacks.len() {
            0 => None,
            1 => Some(callbacks[0].clone()),
            _ => {
                self.ident.span().unstable()
                .error("Multiple functions tagged with receive. Only one permitted per zome.")
                .emit();
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
    fn test_extract_genesis_smoke_test() {
    	let module: syn::ItemMod = parse_quote!{
    		mod zome {
    			#[genesis]
			    fn genisis() {
			        Ok(())
			    }
    		}
    	};
    	let _ = module.extract_zome();
    }

    #[test]
    fn test_extract_single_trait() {
    	let module: syn::ItemMod = parse_quote!{
    		mod zome {    			
    			#[genesis]
			    fn genisis() {
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
    	expected_traits.insert("trait_name".to_string(), TraitFns{functions: vec!["a_fn".to_string()]});
    	assert_eq!{
    		zome_def.traits,
    		expected_traits
    	}
    }

    #[test]
    fn test_multi_function_multi_traits() {
    	let module: syn::ItemMod = parse_quote!{
    		mod zome {    			
    			#[genesis]
			    fn genisis() {
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
    	expected_traits.insert("trait1".to_string(), TraitFns{functions: vec!["a_fn".to_string()]});
    	expected_traits.insert("trait2".to_string(), TraitFns{functions: vec!["a_fn".to_string(), "b_fn".to_string()]});
    	expected_traits.insert("trait3".to_string(), TraitFns{functions: vec!["b_fn".to_string()]});
    	
    	assert_eq!{
    		zome_def.traits,
    		expected_traits
    	}
    }

    #[test]
    fn test_extract_function_params_and_return() {
    	let module: syn::ItemMod = parse_quote!{
    		mod zome {    			
    			#[genesis]
			    fn genisis() {
			        Ok(())
			    }

    			#[zome_fn("test_trait")]
			    fn a_fn(param1: i32, param2: String, param3: bool) -> String {
			        "test".into()
			    }
    		}
    	};
    	let zome_def = module.extract_zome();

    	assert_eq!{
    		zome_def.zome_fns.first().unwrap().declaration,
    		FnDeclaration{
    			name: "a_fn".to_string(),
    			inputs: vec![
    				FnParameter::new("param1", "i32"),
      				FnParameter::new("param2", "String"),
    				FnParameter::new("param3", "bool"),
    			],
    			outputs: vec![
    			    FnParameter::new("result", "String"),
    			],
    		}
    	}
    }

    #[test]
    fn test_extract_function_with_generic_return() {
        let module: syn::ItemMod = parse_quote!{
            mod zome {              
                #[genesis]
                fn genisis() {
                    Ok(())
                }

                #[zome_fn("hc_public")]
                fn a_fn() -> ZomeApiResult<String> {
                    Ok("test".into())
                }
            }
        };
        let zome_def = module.extract_zome();

        assert_eq!{
            zome_def.zome_fns.first().unwrap().declaration,
            FnDeclaration{
                name: "a_fn".to_string(),
                inputs: vec![],
                outputs: vec![
                    FnParameter::new("result", "ZomeApiResult<String>"),
                ],
            }
        }
    }

    #[test]
    fn test_single_entry() {
    	let module: syn::ItemMod = parse_quote!{
    		mod zome {    			
    			#[genesis]
			    fn genisis() {
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
    	assert_eq!{
    		zome_def.entry_def_fns.len(),
    		1
    	}
    }

    #[test]
    fn test_extra_code_in_module() {
    	let module: syn::ItemMod = parse_quote!{
    		mod zome {    			
    			#[genesis]
			    fn genisis() {
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
    	
    	assert_eq!{
    		zome_def.extra.len(),
    		3
    	}
    }

    #[test]
    fn test_no_receive_callback() {
        let module: syn::ItemMod = parse_quote!{
            mod zome {              
                #[genesis]
                fn genisis() {
                    Ok(())
                }
            }
        };
        let zome_def = module.extract_zome();
        assert!(zome_def.receive_callback.is_none())

    }

    #[test]
    fn test_receive_callback() {
        let module: syn::ItemMod = parse_quote!{
            mod zome {              
                #[genesis]
                fn genisis() {
                    Ok(())
                }

                #[receive]
                fn receive() {
                    Ok(())
                }
            }
        };
        let zome_def = module.extract_zome();
        assert!(zome_def.receive_callback.is_some())
    }
}
