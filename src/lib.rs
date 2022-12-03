mod hideable;

use hideable::expand_derive_hideable;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro that automatically implements the Hideable trait. 
/// This adds the hide_fields() fn to whatever you decided to put it on.
/// 
/// NOTE: This macro assumes you have implemented Serialize & Clone on all of the field types.
#[proc_macro_derive(Hideable, attributes(mark))]
pub fn hideable(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);
    match expand_derive_hideable(ident, data) {
        Ok(tokenstream) =>  tokenstream.into(),//panic!("{}", tokenstream.to_string()),
        Err(e) => e.into_compile_error().into(),
    }
}