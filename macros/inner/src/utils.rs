use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, Expr, ExprField, FnArg, Pat, Result};

pub(crate) fn get_fn_arg_names(fn_inputs: &Punctuated<FnArg, Comma>) -> impl Iterator<Item = Result<Ident>> + '_ {
    // TODO(appcypher): Improve error message
    fn_inputs.iter().map(|arg| match arg {
        FnArg::Typed(arg) => match &*arg.pat {
            Pat::Ident(ident) => Ok(ident.ident.clone()),
            _ => Err(syn::Error::new(
                arg.pat.span(),
                "Expected an identifier for the argument",
            )),
        },
        FnArg::Receiver(arg) => match arg.reference {
            Some(_) => Ok(format_ident!("self")),
            None => Err(syn::Error::new(arg.span(), "Expected a reference for the argument")),
        },
    })
}

pub(crate) fn fn_call(fn_inputs: &Punctuated<FnArg, Comma>, fn_name: &Ident) -> TokenStream {
    let fn_arg_names = get_fn_arg_names(fn_inputs).collect::<Result<Vec<_>>>().unwrap();

    if fn_arg_names.iter().any(|arg| arg == &format_ident!("self")) {
        quote! { Self::#fn_name ( #(#fn_arg_names),* ) }
    } else {
        quote! { #fn_name ( #(#fn_arg_names),* ) }
    }
}
