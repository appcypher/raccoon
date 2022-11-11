#![allow(unused_imports)]
extern crate self as raccoon_proc_macros;

mod backtrack;
mod memoize;
mod third_party {
    pub use sha3;
}

use backtrack::BacktrackOptions;
use memoize::MemoizeOptions;
use proc_macro::TokenStream;
pub(crate) use third_party::*;

#[proc_macro_attribute]
pub fn backtrack(attr: TokenStream, item: TokenStream) -> TokenStream {
    let options: BacktrackOptions = syn::parse(attr).unwrap();
    let fn_tree = syn::parse_macro_input!(item as syn::ItemFn);
    backtrack::generate(options, &fn_tree).into()
}

#[proc_macro_attribute]
pub fn memoize(attr: TokenStream, item: TokenStream) -> TokenStream {
    let options: MemoizeOptions = syn::parse(attr).unwrap();
    let fn_tree = syn::parse_macro_input!(item as syn::ItemFn);
    memoize::generate(options, &fn_tree).into()
}
