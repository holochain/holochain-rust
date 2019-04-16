use std::collections::BTreeMap;

use hdk::holochain_core_types::dna::{
    zome::{ZomeTraits},
    fn_declarations::{FnDeclaration, FnParameter, TraitFns},
};

pub type GenesisCallback = syn::Block;
pub type ZomeFunctionCode = syn::Block;
pub type EntryDefCallback = syn::ItemFn;
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
}

pub trait IntoZome {
	fn extract_zome_fns(&self) -> ZomeFunctions;
	fn extract_entry_defs(&self) -> EntryDefCallbacks;
	fn extract_genesis(&self) -> GenesisCallback;
	fn extract_traits(&self) -> ZomeTraits;

	fn extract_zome(&self) -> ZomeCodeDef {
		ZomeCodeDef {
            traits: self.extract_traits(),
            entry_def_fns: self.extract_entry_defs(),
            genesis: self.extract_genesis(),
            zome_fns: self.extract_zome_fns(),
        }
	}
}


static GENESIS_ATTRIBUTE: &str = "genesis";
static ZOME_FN_ATTRIBUTE: &str = "zome_fn";
static ENTRY_DEF_ATTRIBUTE: &str = "entry_def";


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
                        .next()
                        .unwrap()
                        .ident
                        .to_string(),
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
            func.attrs.iter().for_each(|attr| {
                let mlist: syn::MetaList = syn::parse(attr.tts.clone().into()).unwrap();
                mlist.nested.iter().for_each(|e| {
                    if let syn::NestedMeta::Literal(syn::Lit::Str(lit)) = e {
                        let trait_name = lit.value().clone();
                        if let None = acc.get(&trait_name) {
                            acc.insert(trait_name.clone(), TraitFns::new());
                        }
                        acc.get_mut(&trait_name).unwrap().functions.push(func_name.clone());
                    }
                });
            });
	        acc
	    })
	}
}
