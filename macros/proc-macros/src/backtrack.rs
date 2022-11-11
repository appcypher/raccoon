use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    ExprField, ItemFn, Token,
};

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

pub(super) fn generate(options: BacktrackOptions, fn_tree: &ItemFn) -> TokenStream {
    let _ = options.state;

    quote! {
        #fn_tree
    }
}
