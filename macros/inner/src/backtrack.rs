use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    ExprField, ItemFn, Result, Token,
};

use crate::utils;

mod keyword {
    use syn::custom_keyword;

    custom_keyword!(state);
}

pub(super) struct BacktrackOptions {
    state: ExprField,
}

impl Parse for BacktrackOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::state) {
            input.parse::<keyword::state>()?;
            input.parse::<Token![=]>()?;
            Ok(Self { state: input.parse()? })
        } else {
            Err(lookahead.error())
        }
    }
}

fn generate_fn_updated(fn_updated_name: &Ident, fn_tree: &ItemFn) -> TokenStream {
    let fn_vis = &fn_tree.vis;
    let fn_inputs = &fn_tree.sig.inputs;
    let fn_output = &fn_tree.sig.output;
    let fn_block = &fn_tree.block;

    quote! {
        #fn_vis fn #fn_updated_name ( #fn_inputs ) #fn_output #fn_block
    }
}

fn generate_fn_wrapper(fn_updated_name: &Ident, fn_tree: &ItemFn, options: BacktrackOptions) -> TokenStream {
    let fn_name = &fn_tree.sig.ident;
    let fn_attrs = &fn_tree.attrs;
    let fn_vis = &fn_tree.vis;
    let fn_inputs = &fn_tree.sig.inputs;
    let fn_output = &fn_tree.sig.output;
    let state = options.state;
    let fn_call = utils::fn_call(fn_inputs, fn_updated_name);

    quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name ( #fn_inputs ) #fn_output {
            let mut __backtrack_state = #state.clone();
            #fn_call.or_else(|| {
                #state = __backtrack_state;
                None
            })
        }
    }
}

pub(super) fn generate(options: BacktrackOptions, fn_tree: &ItemFn) -> TokenStream {
    let fn_updated_name = &format_ident!("__backtrack_original_{}", fn_tree.sig.ident);
    let fn_updated = generate_fn_updated(fn_updated_name, fn_tree);
    let fn_wrapper = generate_fn_wrapper(fn_updated_name, fn_tree, options);

    quote! {
        #fn_updated
        #fn_wrapper
    }
}
