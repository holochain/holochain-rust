use std::convert::TryFrom;

use crate::into_zome::IntoZome;
use hdk::holochain_core_types::dna::zome::ZomeTraits;
use proc_macro2::{Ident, Span, TokenStream};

pub type InitCallback = syn::Block;
pub type ZomeFunctionCode = syn::Block;
pub type EntryDefCallback = syn::ItemFn;

#[derive(Clone, PartialEq, Debug)]
pub struct ReceiveCallback {
    pub from_param: Ident,
    pub message_param: Ident,
    pub code: syn::Block,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ValidateAgentCallback {
    pub validation_data_param: Ident,
    pub code: syn::Block,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FnParameter {
    pub ident: Ident,
    pub ty: syn::TypePath,
}

impl FnParameter {
    pub fn new(ident: Ident, ty: syn::TypePath) -> Self {
        FnParameter { ident, ty }
    }

    pub fn new_from_ident_str(ident_str: &str, ty: syn::TypePath) -> Self {
        FnParameter {
            ident: Ident::new(ident_str, Span::call_site()),
            ty,
        }
    }

    pub fn new_from_str(ident_str: &str, ty_str: &str) -> Self {
        let ty: syn::TypePath = syn::parse_str(ty_str).unwrap();
        FnParameter {
            ident: Ident::new(ident_str, Span::call_site()),
            ty,
        }
    }
}

impl From<FnParameter> for syn::Field {
    fn from(param: FnParameter) -> Self {
        syn::Field {
            attrs: Vec::new(),
            ident: Some(param.ident),
            ty: syn::Type::Path(param.ty),
            vis: syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token![pub](Span::call_site()),
            }),
            colon_token: Some(syn::Token![:](proc_macro2::Span::call_site())),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct FnDeclaration {
    pub name: String,
    pub inputs: Vec<FnParameter>,
    pub output: syn::ReturnType,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ZomeFunction {
    pub declaration: FnDeclaration,
    pub code: ZomeFunctionCode,
}

pub type ZomeFunctions = Vec<ZomeFunction>;
pub type EntryDefCallbacks = Vec<EntryDefCallback>;

pub struct ZomeCodeDef {
    pub init: InitCallback,
    pub validate_agent: ValidateAgentCallback,
    pub zome_fns: ZomeFunctions, // receive: ReceiveCallbacks
    pub entry_def_fns: Vec<syn::ItemFn>,
    pub traits: ZomeTraits,
    pub receive_callback: Option<ReceiveCallback>,
    pub extra: Vec<syn::Item>, // extra stuff to be added as is to the zome code
}

// use this to convert from the tagged #[zome] module into a ZomeCodeDef struct

impl TryFrom<TokenStream> for ZomeCodeDef {
    type Error = syn::Error;

    fn try_from(input: TokenStream) -> Result<Self, Self::Error> {
        let module: syn::ItemMod = syn::parse(input.into())?;
        Ok(module.extract_zome())
    }
}
