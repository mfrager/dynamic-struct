use core::convert::TryFrom;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Fields, Index, ItemStruct, WhereClause};

use crate::attribute_helpers::contains_skip;

pub fn struct_ser(input: &ItemStruct) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.map_or_else(
        || WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        },
        Clone::clone,
    );
    let mut body = TokenStream2::new();
    match &input.fields {
        Fields::Named(fields) => {
            let mut field_index: usize = 0;
            for field in &fields.named {
                if contains_skip(&field.attrs) {
                    continue;
                }
                let field_name = field.ident.as_ref().unwrap();
                let delta = quote! {
                    CustomSerialize::push_node(&self.#field_name, builder, #field_index)?;
                    CustomSerialize::serialize(&self.#field_name, builder)?;
                    CustomSerialize::pop_node(&self.#field_name, builder)?;
                };
                field_index += 1;
                body.extend(delta);
                let field_type = &field.ty;
                where_clause.predicates.push(
                    syn::parse2(quote! {
                        #field_type: CustomSerialize
                    })
                    .unwrap(),
                );
            }
        }
        Fields::Unnamed(fields) => {
            for field_idx in 0..fields.unnamed.len() {
                let field_idx = Index {
                    index: u32::try_from(field_idx).expect("up to 2^32 fields are supported"),
                    span: Span::call_site(),
                };
                let delta = quote! {
                    CustomSerialize::push_node(&self.#field_idx, builder, #field_idx)?;
                    CustomSerialize::serialize(&self.#field_idx, builder)?;
                    CustomSerialize::pop_node(&self.#field_idx, builder)?;
                };
                body.extend(delta);
            }
        }
        Fields::Unit => {}
    }
    println!("{:?}", body.to_string());
    Ok(quote! {
        impl #impl_generics CustomSerialize for #name #ty_generics #where_clause {
            fn serialize<B: Build>(&self, builder: &mut B) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                if builder.is_root() {
                    builder.stack_push(0)?;
                }
                builder.build(None)?;
                #body
                Ok(())
            }
        }
    })
}
