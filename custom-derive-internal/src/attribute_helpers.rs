use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Meta, NestedMeta, Path};

pub fn contains_skip(attrs: &[Attribute]) -> bool {
    for attr in attrs.iter() {
        if let Ok(Meta::Path(path)) = attr.parse_meta() {
            if path.to_token_stream().to_string().as_str() == "custom_skip" {
                return true;
            }
        }
    }
    false
}
