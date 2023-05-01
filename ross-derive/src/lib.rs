mod table;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};
use quote::quote;


#[proc_macro_derive(Table, attributes(field))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    crate::table::expand_table_derive(&mut input)
    .unwrap_or_else(to_compile_errors)
    .into()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}