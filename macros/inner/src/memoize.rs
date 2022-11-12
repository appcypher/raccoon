use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Expr, ExprField, FnArg, Ident, ItemFn, Pat, Result, Token,
};

use crate::utils;

mod keyword {
    use syn::custom_keyword;

    custom_keyword!(cache);
    custom_keyword!(key_extension);
}

pub(super) enum MemoizeOption {
    Cache(ExprField),
    KeyExtra(ExprField),
}

pub(super) struct MemoizeOptions {
    cache: ExprField,
    key_extension: Option<ExprField>,
}

impl Parse for MemoizeOption {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::cache) {
            input.parse::<keyword::cache>()?;
            input.parse::<Token![=]>()?;
            Ok(Self::Cache(input.parse()?))
        } else if lookahead.peek(keyword::key_extension) {
            input.parse::<keyword::key_extension>()?;
            input.parse::<Token![=]>()?;
            Ok(Self::KeyExtra(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for MemoizeOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let options = input.parse_terminated::<MemoizeOption, Token![,]>(MemoizeOption::parse)?;
        options
            .iter()
            .find(|option| matches!(option, MemoizeOption::Cache(_)))
            .ok_or_else(|| {
                syn::Error::new(
                    input.span(),
                    "Expected at least one `cache` option for the `memoize` macro",
                )
            })?;

        // Construct memoize options.
        let options = options
            .into_iter()
            .map(|option| match option {
                MemoizeOption::Cache(cache) => (Some(cache), None),
                MemoizeOption::KeyExtra(key_extension) => (None, Some(key_extension)),
            })
            .fold((None, None), |(c1, k1), (c2, k2)| (c1.or(c2), k1.or(k2)));

        Ok(Self {
            cache: options.0.unwrap(),
            key_extension: options.1,
        })
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

fn exclude_cache_from_args<'a>(
    fn_inputs: &'a Punctuated<FnArg, Comma>,
    cache: &'a ExprField,
) -> impl Iterator<Item = Result<Ident>> + 'a {
    // TODO(appcypher):
    // - Optimize. Another call to get_fn_arg_names.
    // - Fix cache being always an ExprField. Should support ExprPath as well.
    utils::get_fn_arg_names(fn_inputs).filter(move |arg| match arg {
        Ok(arg) => match &*cache.base {
            Expr::Path(path) => match path.path.get_ident() {
                Some(ident) => arg != ident,
                None => true,
            },
            _ => true,
        },
        Err(_) => true,
    })
}

fn generate_fn_wrapper(fn_updated_name: &Ident, fn_tree: &ItemFn, options: MemoizeOptions) -> TokenStream {
    let fn_name = &fn_tree.sig.ident;
    let fn_name_str = fn_name.to_string();
    let fn_attrs = &fn_tree.attrs;
    let fn_vis = &fn_tree.vis;
    let fn_inputs = &fn_tree.sig.inputs;
    let fn_output = &fn_tree.sig.output;
    let fn_arg_names_no_cache = exclude_cache_from_args(&fn_tree.sig.inputs, &options.cache)
        .collect::<Result<Vec<_>>>()
        .unwrap();
    let fn_call = utils::fn_call(fn_inputs, fn_updated_name);

    let MemoizeOptions {
        ref cache,
        ref key_extension,
    } = options;

    let hash_input = if key_extension.is_some() {
        quote! { &format!("{:?}/{:?}/{:?}", #fn_name_str, (#(#fn_arg_names_no_cache),*), #key_extension) }
    } else {
        quote! { &format!("{:?}/{:?}", #fn_name_str, (#(#fn_arg_names_no_cache),*)) }
    };

    quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name ( #fn_inputs ) #fn_output {
            use raccoon_macros::Cache;
            use raccoon_macros::third_party::sha3::{Digest, Sha3_256};

            let key: [u8; 32] = {
                let mut hasher = Sha3_256::new();
                hasher.update(#hash_input);
                hasher.finalize().into()
            };

            if let Some(value) = #cache.get(&key) {
                return value.clone();
            }

            let value = #fn_call;
            #cache.insert(key, value.clone());
            value
        }
    }
}

pub(super) fn generate(options: MemoizeOptions, fn_tree: &ItemFn) -> TokenStream {
    let fn_updated_name = &format_ident!("__memoize_original_{}", fn_tree.sig.ident);
    let fn_updated = generate_fn_updated(fn_updated_name, fn_tree);
    let fn_wrapper = generate_fn_wrapper(fn_updated_name, fn_tree, options);

    quote! {
        #fn_updated
        #fn_wrapper
    }
}
