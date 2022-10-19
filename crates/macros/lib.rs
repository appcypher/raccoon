use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn backtrack(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: {}", attr);
    println!("item: {}", item);
    item
}

#[proc_macro_attribute]
pub fn memoize(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: {}", attr);
    println!("item: {}", item);
    item
}
