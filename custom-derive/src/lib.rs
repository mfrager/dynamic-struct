extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{ItemEnum, ItemStruct, ItemUnion};

use custom_derive_internal::*;

#[proc_macro_derive(CustomSerialize, attributes(custom_skip))]
pub fn borsh_serialize(input: TokenStream) -> TokenStream {
    let res = if let Ok(input) = syn::parse::<ItemStruct>(input.clone()) {
        struct_ser(&input)
    } else if let Ok(_input) = syn::parse::<ItemEnum>(input.clone()) {
        //enum_ser(&input, cratename)
        unreachable!()
    } else if let Ok(_input) = syn::parse::<ItemUnion>(input) {
        //union_ser(&input, cratename)
        unreachable!()
    } else {
        // Derive macros can only be defined on structs, enums, and unions.
        unreachable!()
    };
    TokenStream::from(match res {
        Ok(res) => res,
        Err(err) => err.to_compile_error(),
    })
}
